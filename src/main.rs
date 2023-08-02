// fn main() {
//     println!("Hello, world!");
// }

use axum::{response::{Html, Redirect, Response, IntoResponse}, routing::get, Router};
use axum::extract::{Query};
use std::collections::HashMap;
use rspotify::{
    model::{TimeRange, Market, PlayableItem::{Episode, Track}}, prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token,
};

const CLIENT_ID: &str = "7b835c32774d4660b3d3e394b0deb8ac";
const CLIENT_SECRET: &str = "have a look";
// static TOKEN: &'static str= "";


fn init_spotify() -> AuthCodeSpotify {
    let config = Config {
        token_cached: true,
        ..Default::default()
    };

    // Please notice that protocol of redirect_uri, make sure it's http (or
    // https). It will fail if you mix them up.
    let oauth = OAuth {
        scopes: scopes!(
            "user-read-currently-playing",
            "user-read-playback-state"
        ),
        
        redirect_uri: "http://localhost:3000/callback".to_owned(),
        ..Default::default()
    };

    let creds = Credentials::new(CLIENT_ID, CLIENT_SECRET);
    return AuthCodeSpotify::with_config(creds, oauth, config)
}


async fn auth() -> Redirect {
    let spotify = init_spotify();

    let auth_url = spotify.get_authorize_url(true).unwrap();
    
    return Redirect::to(auth_url.as_str())
}

async fn callback(Query(params): Query<HashMap<String, String>>) -> Response {
    let spotify = init_spotify();
    
    match spotify.request_token(&params["code"]).await {
        Ok(_) => {
            // info!("Requested user token successfully");
            // AppResponse::Redirect(Redirect::to("/"))
            
            return Redirect::to("/").into_response()
        }
        Err(err) => {
            // error!("Failed to get user token: {:?}", err);
            // let mut context = HashMap::new();
            // context.insert("err_msg", "Failed to get token!");
            return Html(err.to_string()).into_response()
            // AppResponse::Template(Template::render("error", context))
        }
    }
    
    // return Html(format!("Hello, world! {:?}", params))

}

async fn home() -> Html<String> {
    let token = Token::from_cache(".spotify_token_cache.json").unwrap();
    let spotify = AuthCodeSpotify::from_token(token);

    let state = spotify.current_playback(None, None::<Vec<_>>).await.unwrap();
    
    match state {
        Some(playing) => {
            match playing.item {
                Some(item) => match item {
                    Track(track) => return Html(format!("Currently playing: {} - {} by {}", track.album.name, track.name, track.artists[0].name)),
                    Episode(episode) => return Html(format!("Currently playing: {}", episode.name)),
                },
                None => return Html("nothing is playing".to_string()),
            }
        },
        None => return Html("nothing is playing".to_string()),
    }

    
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route(
            "/",
            get(home),
        )
        .route(
            "/auth",
            get(auth),
        )
        .route(
            "/callback",
            get(callback),
        );


        println!("Web server running at http://localhost:3000/");
    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
