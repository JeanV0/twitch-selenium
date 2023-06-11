mod twitch;

use std::{env, sync::{Mutex, Arc}, time::Duration, thread::sleep};

use tokio::sync::{mpsc, mpsc::{Receiver, Sender}};

use thirtyfour::prelude::WebDriverResult;
use dotenv::dotenv;
use twitch::TwitchClient;

async fn controller(thread: i64,tx: Sender<String>) -> WebDriverResult<i64> {
    let mut client: TwitchClient = TwitchClient::new(thread.to_string()).await?;
    let channel = env::var("CHANNEL").unwrap().parse().unwrap();
    let counter = client.watch_ads(channel).await?;
    
    let duration_sleep: u64 = env::var("SLEEP").unwrap().parse().unwrap();
    sleep(Duration::from_secs(duration_sleep));

    if let Err(_) = tx.send(String::from("Nao deu merda")).await {
        println!("Tarefa terminada");
    } 
    Ok(counter)
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    dotenv().ok();
    let thread: i64 = env::var("THREAD").unwrap().parse().unwrap();
    let mut processes: Vec<Arc<Mutex<Receiver<String>>>> = Vec::new();

    for process in 0..thread {
        let (tx, rx) = mpsc::channel::<String>(100);
        processes.push(Arc::new(Mutex::new(rx)));
        tokio::spawn(async move {
            let mut counter_actual: i64 = 0;
            let counter_target: i64 = env::var("QUANTITY").unwrap().parse().unwrap();
            while counter_target > counter_actual {
                counter_actual += match controller(thread, tx.clone()).await {
                    Ok(counter) => {
                        println!("thread {} ads assistidos: {}",process,counter_actual);
                        counter
                    },
                    Err(err) => {
                        println!("Deu erro\n{}",err);
                        0
                    },
                };
            };
        });
        sleep(Duration::from_secs(10)); 
    }

    for process in processes {
        while let Some(_) = process.lock().unwrap().recv().await  {}
    }
    
    Ok(())
}
