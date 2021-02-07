use reqwest::StatusCode;
use std::{convert::Infallible, env, net::SocketAddr};
use teloxide::{
    dispatching::update_listeners,
    prelude::*,
    types::{InlineQuery, InlineQueryResult, InlineQueryResultCachedVoice},
};
use tokio::sync::mpsc;
use warp::Filter;

const VOICES: &[&[&str; 3]] = &[
    &[
        "0",
        "AwACAgEAAxkBAAEI24RgH1Dpmq_o3Bg7aj-rm5jr34_2TQACJQEAAna34USKqpTxoLb5Rh4E",
        "Ed",
    ],
    &[
        "1",
        "AwACAgEAAxkBAAEI24ZgH1H6mVVyvX1GsNPoV2r4bFJXvgACJgEAAna34US-0twcFBb9gR4E",
        "Ivan",
    ],
    &[
        "2",
        "AwACAgEAAxkBAAEI24hgH1KjxruMznTwyKnet1kMg_LOoQACDQEAAiwl-UQYq-d0Ei354B4E",
        "Matt",
    ],
];

#[tokio::main]
async fn main() {
    run().await;
}

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Cannot process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn webhook<'a>(bot: Bot) -> impl update_listeners::UpdateListener<Infallible> {
    // Heroku defines auto defines a port value
    let teloxide_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN env variable missing");
    let port: u16 = env::var("PORT")
        .expect("PORT env variable missing")
        .parse()
        .expect("PORT value to be integer");
    // Heroku host example .: "heroku-ping-pong-bot.herokuapp.com"
    let host = env::var("HOST").expect("have HOST env variable");
    let path = format!("bot{}", teloxide_token);
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url)
        .send()
        .await
        .expect("Cannot setup a webhook");

    let (tx, rx) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            let try_parse = match serde_json::from_str(&json.to_string()) {
                Ok(update) => Ok(update),
                Err(error) => {
                    log::error!(
                        "Cannot parse an update.\nError: {:?}\nValue: {}\n\
                       This is a bug in teloxide, please open an issue here: \
                       https://github.com/teloxide/teloxide/issues.",
                        error,
                        json
                    );
                    Err(error)
                }
            };
            if let Ok(update) = try_parse {
                tx.send(Ok(update))
                    .expect("Cannot send an incoming update from the webhook")
            }

            StatusCode::OK
        })
        .recover(handle_rejection);

    let serve = warp::serve(server);
    let address = format!("0.0.0.0:{}", port);

    tokio::spawn(serve.run(address.parse::<SocketAddr>().unwrap()));
    rx
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting @heypeterbot…");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .inline_queries_handler(|rx: DispatcherHandlerRx<InlineQuery>| {
            rx.for_each_concurrent(None, |query| async move {
                let query_text = query.update.query.trim();
                query
                    .bot
                    .answer_inline_query(
                        query.update.id,
                        if query_text.is_empty() {
                            VOICES
                                .iter()
                                .map(|&&voice| {
                                    InlineQueryResult::CachedVoice(
                                        InlineQueryResultCachedVoice::new(
                                            voice[0], voice[1], voice[2],
                                        ),
                                    )
                                })
                                .collect::<Vec<InlineQueryResult>>()
                        } else {
                            VOICES
                                .iter()
                                .filter_map(|&&voice| {
                                    if voice[2].to_lowercase().contains(&query_text.to_lowercase())
                                    {
                                        Some(InlineQueryResult::CachedVoice(
                                            InlineQueryResultCachedVoice::new(
                                                voice[0], voice[1], voice[2],
                                            ),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<InlineQueryResult>>()
                        },
                    )
                    .send()
                    .await
                    .log_on_error()
                    .await;
            })
        })
        .dispatch()
        .await;
}
