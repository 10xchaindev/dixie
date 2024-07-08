use rocket::{
    async_trait,
    http::
        uri::{Authority, Host, Origin}
    ,
    request::{FromRequest, Outcome},
};

use crate::core::config::Config;

pub struct Uri<'a> {
    pub origin: Origin<'a>,
    pub host: Option<Host<'a>>,
}

impl<'a> ToString for Uri<'a> {
    fn to_string(&self) -> String {
        match &self.host {
            Some(host) => {
                let host = &host.to_string();
                let path = &self.origin.path().to_string();
                let url = String::from("https://") + host + path;
                url
            }
            None => panic!("invalid url. provide a valid host"),
        }
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for Uri<'r> {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let origin = request.uri().clone();
        let host = request.host();

        if let Some(host) = host {
            Outcome::Success(Uri {
                origin,
                host: Some(host.clone()),
            })
        } else {
            let config = request.rocket().state::<Config>().unwrap();
            Outcome::Success(Uri {
                origin,
                host: Some(Host::new(Authority::parse(&config.address).unwrap())),
            })
        }
    }
}
