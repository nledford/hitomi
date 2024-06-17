use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

pub async fn mix_random_data() -> Result<()> {
    #[derive(Deserialize)]
    struct Csprng {
        #[serde(alias = "Data")]
        data: String,
    }

    match reqwest::get("https://csprng.xyz/v1/api").await {
        Ok(resp) => {
            let contents = resp.text().await?;

            match serde_json::from_str::<Csprng>(&contents) {
                Ok(csprng) => {
                    tokio::fs::write("/dev/urandom", csprng.data).await?;
                    Ok(())
                }
                Err(err) => {
                    Err(err).with_context(|| {
                        format!("Unable to deserialise response. Body was: \"{}\"", contents)
                    })
                }
            }
        }
        Err(err) => {
            Err(anyhow!("An error occurred while attempting to fetch random data: {err}"))
        }
    }
}