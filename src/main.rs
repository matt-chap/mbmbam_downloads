extern crate clap;

use clap::Parser;
use log::{info, warn};
use regex::Regex;
use std::path::Path;

use crabler::*;

const INDEX_URL: &'static str = "https://maximumfun.org/transcripts/my-brother-my-brother-and-me/transcript-mbmbam-001-gettin-beebed/";

#[derive(Parser, Debug)]
#[clap(version = "0.1", author = "Potato")]
struct CliOpts {
    #[clap(long = "directory", short = 'd', default_value = ".")]
    directory: String,
    #[clap(long = "threads", short = 't', default_value_t = 8)]
    threads: usize,
}

#[derive(WebScraper)]
#[on_response(on_response)]
#[on_html("#transcript-buttons > a[href]", entry_handler)]
#[on_html("#transcript-next > a[href]", index_handler)]
struct Scraper {
    index_href_re: Regex,
    directory: String
}

impl Scraper {
    async fn on_response(&self, response: Response) -> Result<()> {
        if let Some(destination) = response.download_destination {
            if response.url.ends_with(".pdf") && response.status == 200 {
                warn!("Finished downloading {} -> {}", response.url, destination);
            }
        }

        Ok(())
    }
    
    async fn index_handler(&self, mut response: Response, a: Element) -> Result<()> {
        if let Some(href) = a.attr("href") {
            //info!("Found some href {}", href);
            if let Some(rel) = a.attr("rel") {
                if self.index_href_re.is_match(&href[..]) && rel == "next" {
                    //info!("Navigating to {}", href);
                    response.navigate(href).await?;
                };
            }
        }

        Ok(())
    }

    async fn entry_handler(&self, mut response: Response, a: Element) -> Result<()> {
        if let Some(href) = a.attr("href") {
            
            if href.ends_with(".pdf") {
                //info!("PDF {}", href);
                let fname = href.rsplit_once("/").unwrap().1;
                info!("Name {}", fname);
                let p = Path::new(&self.directory).join(Path::new(&fname));
                let destination = p.to_string_lossy();

                if !p.exists() {
                    //warn!("Downloading {}", destination);
                    response
                        .download_file(href, destination.to_string())
                        .await?;
                } else {
                    info!("Skipping exist file {}", destination);
                }
            };
        }

        Ok(())
    }

}

#[async_std::main]
async fn main() -> Result<()> {
    femme::with_level(log::LevelFilter::Info);

    let opts: CliOpts = CliOpts::parse();

    let index_href_re = Regex::new(r".*transcript-mbmbam-.*").unwrap();
    let directory = opts.directory.clone();
    let scraper = Scraper {
        index_href_re,
        directory
    };

    scraper
        .run(
            Opts::new()
                .with_urls(vec![INDEX_URL])
                .with_threads(opts.threads),
        )
        .await
}