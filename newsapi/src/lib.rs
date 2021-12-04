use serde::Deserialize;
use url::Url;

const BASE_URL: &str = "https://newsapi.org/v2";

#[derive(thiserror::Error, Debug)]
pub enum NewsApiError {
    #[error("Failed fetching articles")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed parsing response into json")]
    ArticleParseFailed(reqwest::Error),
    #[error("URL parsing failed")]
    UrlParsing(#[from] url::ParseError),
    #[error("Request Failed {0}")]
    BadRequest(&'static str),
}

#[derive(Debug, Deserialize)]
pub struct NewsApiResponse {
    status: String,
    pub articles: Vec<Article>,
    code: Option<String>,
}

impl NewsApiResponse {
    pub fn articles(&self) -> &Vec<Article> {
        &self.articles
    }
}

#[derive(Debug, Deserialize)]
pub struct Article {
    title: String,
    url: String,
    description: Option<String>,
}

impl Article {
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn desc(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

pub enum Endpoint {
    TopHeadLines,
}

impl ToString for Endpoint {
    fn to_string(&self) -> String {
        match self {
            Self::TopHeadLines => "top-headlines".to_string(),
        }
    }
}

pub enum Country {
    Us,
}

impl ToString for Country {
    fn to_string(&self) -> String {
        match self {
            Self::Us => "us".to_string(),
        }
    }
}

pub struct NewsAPI {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
}

impl NewsAPI {
    pub fn new(api_key: &str) -> NewsAPI {
        NewsAPI {
            api_key: api_key.to_string(),
            endpoint: Endpoint::TopHeadLines,
            country: Country::Us,
        }
    }
    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut NewsAPI {
        self.endpoint = endpoint;
        self
    }

    pub fn country(&mut self, country: Country) -> &mut NewsAPI {
        self.country = country;
        self
    }

    fn prepare_url(&self) -> Result<String, NewsApiError> {
        let mut url = Url::parse(BASE_URL)?;
        url.path_segments_mut()
            .unwrap()
            .push(&self.endpoint.to_string());
        let country = format!("country={}", self.country.to_string());
        url.set_query(Some(&country));
        Ok(url.to_string())
    }

    pub async fn fetch(&self) -> Result<NewsApiResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let response = reqwest::Client::new()
            .get(url)
            .header("Authorization", &self.api_key)
            .send()
            .await?
            .json::<NewsApiResponse>()
            .await?;
        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_response_err(response.code)),
        }
    }

    #[cfg(feature = "blocking")]
    pub fn fetch_blocking(&self) -> Result<NewsApiResponse, NewsApiError> {
        let url = self.prepare_url()?;
        let response = reqwest::blocking::Client::new()
            .get(url)
            .header("Authorization", &self.api_key)
            .send()?
            .json::<NewsApiResponse>()?;
        match response.status.as_str() {
            "ok" => return Ok(response),
            _ => return Err(map_response_err(response.code)),
        }
    }
}

fn map_response_err(code: Option<String>) -> NewsApiError {
    if let Some(code) = code {
        match code.as_str() {
            "apiKeyDisabled" => NewsApiError::BadRequest("Your API key has been disabled"),
            _ => NewsApiError::BadRequest("Unknown Error"),
        }
    } else {
        NewsApiError::BadRequest("Unknown Error")
    }
}
