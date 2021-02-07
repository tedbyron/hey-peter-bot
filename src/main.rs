use teloxide::{
    prelude::*,
    types::{InlineQuery, InlineQueryResult, InlineQueryResultCachedVoice},
};

const AUDIO: &[&[&str; 3]] = &[
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

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting @heypeterbot…");

    let bot = Bot::from_env();

    Dispatcher::new(bot)
        .inline_queries_handler(|rx: DispatcherHandlerRx<InlineQuery>| {
            rx.for_each_concurrent(None, |query| async move {
                query
                    .bot
                    .answer_inline_query(
                        query.update.id,
                        AUDIO
                            .iter()
                            .map(|&&voice| {
                                InlineQueryResult::CachedVoice(InlineQueryResultCachedVoice::new(
                                    voice[0], voice[1], voice[2],
                                ))
                            })
                            .collect::<Vec<InlineQueryResult>>(),
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
