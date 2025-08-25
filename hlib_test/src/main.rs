mod test_json;
use std::time::Duration;

use fruits_engine::utils::thread_pool::ThreadPool;

fn main() {
    // test_json::test_serialization();

    let pool = ThreadPool::new(1);

    println!("before scope");

    pool.scope(|s| {
        println!("scope start");

        s.push_job_unhandled(|| {
            std::thread::sleep(Duration::from_secs_f32(1.0));
            println!("job 1");
        });

        s.push_job_unhandled(|| {
            std::thread::sleep(Duration::from_secs_f32(1.0));
            println!("job 2");
        });

        println!("scope end");
    });
    
    println!("after scope");
}

