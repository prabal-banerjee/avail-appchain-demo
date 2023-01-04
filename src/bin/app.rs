use avail_subxt::primitives::AppUncheckedExtrinsic;
use serde::Deserialize;
use nalgebra::Matrix2;


#[derive(Deserialize, Debug)]
struct Status {
    block_num: u32,
    #[allow(dead_code)]
    confidence: f32,
    app_id: u32
}

#[derive(Deserialize, Debug)]
struct BlockData {
    #[allow(dead_code)]
    block: u32,
    extrinsics: Vec<AppUncheckedExtrinsic>
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Assume cmd line arg contains port number
    // TODO: stricter check on cmd line args
    if args.len() != 2 {
        println!("Error: Provide port number as a command line argument");
        return;
    }
    let port = &args[1];

    let status_url = format!("http://127.0.0.1:{}/v1/status", port);
    let response = minreq::get(&status_url).send().unwrap();
    match response.status_code {
        200 => {
            let status: Status = response.json().unwrap();
            println!("{:?}", status);
            assert_eq!(status.app_id, 1);
        }
        _ => {
            println!("Error: Could not reach endpoint correctly. Check LC!");
            return;
        }
    }

    // let appdata_url = format!("http://127.0.0.1:7000/v1/appdata/{}", status.block_num-1);
    let mut block_num = 1;
    let mut product = Matrix2::<u32>::identity();
    loop {
        let appdata_url = format!("http://127.0.0.1:{}/v1/appdata/{}", port, block_num);
        let response = minreq::get(appdata_url)
            .send()
            .unwrap();
            
        match response.status_code {
            404 => {    // Not found. Go for next block.    
                let status: Status = minreq::get(&status_url).send().unwrap().json().unwrap();

                if status.block_num < block_num {
                    println!("Waiting for new block to be proposed. Last block: {}", block_num);
                    std::thread::sleep(std::time::Duration::from_secs(20));
                    continue;
                }
                println!("No data found in block {}", block_num);
            },  
            401 => {    // Processing. Wait for next block.
                println!("Processing block {}. Waiting for block to be processed.", block_num);
                std::thread::sleep(std::time::Duration::from_secs(20));
                continue;
            },
            200 => {    // Found. Process block data. 
                let block_data: BlockData = response.json().unwrap();
                product = block_data.extrinsics
                    .iter()
                    .fold(
                        product, 
                        |product, ext| product * &unwrap_extrinsic(ext)
                    );
                println!("Product matrix: {}", product);
                product = product.map(|el| el%19);
                println!("Reduced matrix: {}", product);
            },
            _ => {      // Exception. Exit program. 
                println!("Block number {} data fetch error. Error code: {}", block_num, response.status_code);
                return;
            }
        };

        block_num += 1;
    }

}

fn unwrap_extrinsic (ext: &AppUncheckedExtrinsic) -> Matrix2<u32> {
    if let avail_subxt::api::runtime_types::da_runtime::Call::DataAvailability(call) = &ext.function {
        if let avail_subxt::api::runtime_types::da_control::pallet::Call::submit_data{ data } = call {
            let str = String::from_utf8(data.clone().0).unwrap();
            let deserialized_matrix: Matrix2<u32> = serde_json::from_str(&str).unwrap();
            println!("{:?}", deserialized_matrix);
            return deserialized_matrix;
        }
        
    }
    println!("Extrinsic unwrapping to matrix failed!");
    Matrix2::<u32>::identity()
}