use anyhow::Result;
use avail_subxt::{
	api::{
		self,
		runtime_types::{
			da_control::pallet::Call as DaCall, frame_support::storage::bounded_vec::BoundedVec,
		},
	},
	build_client,
	primitives::AvailExtrinsicParams,
	Call, Opts,
};
use structopt::StructOpt;
use subxt::tx::PairSigner;
use async_std::task;
use async_std::task::JoinHandle;
use sp_keyring::sr25519::Keyring;
use futures::future;
use nalgebra::Matrix2;
use serde_json;
use rand::{rngs::StdRng, RngCore};
use rand::SeedableRng;

fn main() {
	let accounts: [sp_keyring::sr25519::sr25519::Pair; 6] = [
		Keyring::Alice.pair(), Keyring::Bob.pair(), Keyring::Charlie.pair(),
		Keyring::Dave.pair(), Keyring::Eve.pair(), Keyring::Ferdie.pair()
	];

	loop {
		let mut tasks: Vec<JoinHandle<()>> = Vec::new();
		for i in 0..accounts.len() {
			let account = accounts[i].clone();
			let task = task::spawn(async move {
				let result = fire_transaction(account, i).await;
				match result {
					Ok(_) => println!("Success from thread {}", i),
					Err(e) => println!("Error firing transaction: {:?} from thread {}", e, i)
				}
			});
			tasks.push(task);
		}

		let handler = future::join_all(tasks.into_iter());
		task::block_on(handler);
	}

}

async fn fire_transaction(account: sp_keyring::sr25519::sr25519::Pair, index: usize) -> Result<()> {
	let args = Opts::from_args();
	let client = build_client(args.ws).await?;

	let mut rng = StdRng::from_entropy();
	// let matrix = Matrix2::<i8>::new_random();
	let matrix = Matrix2::<u32>::from_fn(|_,_| (rng.next_u32() % 19) );
	let serialized_matrix = serde_json::to_string(&matrix)?.as_bytes().to_vec();

	let signer = PairSigner::new(account);
	let data_transfer = api::tx()
		.data_availability()
		.submit_data(BoundedVec(serialized_matrix.clone()));
	let extrinsic_params = AvailExtrinsicParams::new_with_app_id(1.into());

	println!("Sending {:?} from thread {}", matrix, index);
	let h = client
		.tx()
		.sign_and_submit_then_watch(&data_transfer, &signer, extrinsic_params)
		.await?
		.wait_for_in_block()
		.await?;

	let submitted_block = client.rpc().block(Some(h.block_hash())).await?.unwrap();

	let matched_xt = submitted_block
		.block
		.extrinsics
		.into_iter()
		.find(|ext| match &ext.function {
			Call::DataAvailability(da_call) => match da_call {
				DaCall::submit_data { data } => data.0 == serialized_matrix,
				_ => false,
			},
			_ => false,
		});

	assert!(matched_xt.is_some(), "Submitted data not found");

	Ok(())
}