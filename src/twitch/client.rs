
use std::{time::Duration, thread::sleep, path::Path, process::{Command, Child, Stdio}, env};

use thirtyfour::{prelude::{WebDriver, WebDriverResult}, DesiredCapabilities, By};
use regex::Regex;

pub struct TwitchClient {
    client: WebDriver,
    process_webdriver: Child,
}

impl TwitchClient {
    pub async fn new(thread: String) -> WebDriverResult<Self>  {
        let mut caps = DesiredCapabilities::chrome();
        
        let chromedriver_path = Path::new("./chromedriver.exe");
        let crx_file = Path::new("./twitchVPN.crx");
        caps.set_disable_gpu()?;
        caps.add_chrome_arg("--user-agent=\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.0.0.0 Safari/537.36\"")?;
        caps.add_chrome_arg("--disable-resize")?;
        caps.add_chrome_arg("--disable-infobars")?;
        caps.add_chrome_arg("--window-size=800,600")?;
        caps.add_chrome_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_chrome_arg("--enable-automation")?;
        caps.add_extension(crx_file)?;
        let args = [
            format!("--port=44{}9",thread),
            "--log-level=OFF".to_owned(),
        ];

        std::thread::sleep(std::time::Duration::from_secs(2));        

        let process_webdriver = Command::new(chromedriver_path)
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

        let client: WebDriver = WebDriver::new(&format!("http://localhost:44{}9", thread), caps).await?;
        client.set_script_timeout(Duration::from_secs(120)).await?;
        client.set_page_load_timeout(Duration::from_secs(120)).await?;

        Ok(Self {
            process_webdriver,
            client
        })
    }

    pub async fn watch_ads(&mut self, channel: String) -> WebDriverResult<i64> {
        // Navigate to https://wikipedia.org.
        let view_ads: bool = env::var("VIEW_ADS").unwrap().parse().unwrap();

        if view_ads {
            let ads = "/html/body/div[1]/div/div[2]/div/main/div[1]/div[3]/div/div/div[2]/div/div[2]/div/div[3]/div/div/div[1]/div[4]/span";
    
            let mut counter: i64 = 0;

            self.client.goto(format!("https://www.twitch.tv/{}", channel)).await?;
            match self.client.find(By::XPath(ads)).await {
                Ok(element) => {
                    println!("{}",element.text().await?);
        
                    if element.text().await?.contains("Ad 1 of") {
                        let re = Regex::new(r"of (\d+)").unwrap();
    
                        if let Some(captures) = re.captures(element.text().await?.as_str()) {
                            if let Some(number) = captures.get(1) {
                                let number_str = number.as_str();
                                let parsed_number: i64 = number_str.parse().unwrap();
                                counter = parsed_number;
                            }
                        }
                    } else if element.text().await?.contains(":") {
                        counter = 1;
                    } 
    
                    println!("Anuncios achados {}",counter);
    
                    let mut conditional: bool = true;
                    while conditional {
                        conditional = match self.client.find(By::XPath(ads)).await {
                            Ok(_) => {
                                sleep(Duration::from_secs(2));
                                true
                            },
                            Err(_) => {
                                false
                            },
                        }
                    };
                    self.process_webdriver.kill()?;
                    return Ok(counter)
                },
                Err(_err) => {
                    println!("Sem anuncio");
                    self.client.clone().quit().await?;
                    self.process_webdriver.kill()?;
                    return Ok(counter)
                },
            }
        } else {
            sleep(Duration::from_secs(1));
            self.client.goto("chrome-extension://omghfjlpggmjjaagoclmmobgdodcjboh/popup/popup.html").await?;

            match self.client.find(By::XPath("/html/body/div[2]")).await {
                Ok(vpn) => {
                    vpn.click().await?;
                    sleep(Duration::from_secs(1));
                    self.client.goto(format!("https://www.twitch.tv/{}", channel)).await?;
                },
                Err(_) => todo!(),
            }

            sleep(Duration::from_secs(100000));
            Ok(0)
        }
    }
}