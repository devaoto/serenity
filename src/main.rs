#![warn(clippy::str_to_string)]

mod commands;

use poise::serenity_prelude as serenity;
use ::serenity::all::RoleId;
use std::{ collections::HashMap, sync::{ Arc, Mutex }, time::Duration };
use std::env;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().expect("Failed to load .env file");

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::vote(),
            commands::getvotes(),
            commands::ping(),
            commands::send_verification()
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(
                Arc::new(poise::EditTracker::for_timespan(Duration::from_secs(3600)))
            ),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey serenity"),
                poise::Prefix::Literal("hey serenity,")
            ],
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!("Got an event in event handler: {:?}", event.snake_case_name());

                match event {
                    serenity::FullEvent::ReactionAdd { add_reaction } => {
                        if add_reaction.message_id == 1278756946886791220 {
                            let guild = add_reaction.guild_id.unwrap();
                            let user_id = add_reaction.user_id.unwrap();
                            let member = guild.member(&_ctx.http, user_id).await?;

                            let role_id = RoleId::new(1278703173237866599);

                            if !member.roles.contains(&role_id) {
                                if let Err(why) = member.add_role(&_ctx.http, role_id).await {
                                    println!("Error adding role: {:?}", why);
                                } else {
                                    let dm = user_id.create_dm_channel(&_ctx.http).await.unwrap();
                                    let _ = dm
                                        .say(&_ctx.http, "You have been verified!").await
                                        .expect("An error occurred sending message, maybe DM off?");
                                }
                            }
                        }
                    }
                    _ => {}
                }

                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework
        ::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    let token = env::var("BOT_TOKEN").expect("Expected a token in the environment");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents).framework(framework).await;

    client.unwrap().start().await.unwrap()
}
