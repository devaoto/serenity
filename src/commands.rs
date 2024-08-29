use poise::CreateReply;
use serenity::all::{ CreateEmbed, ReactionType };

use crate::{ Context, Error };
use std::time::Instant;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] #[autocomplete = "poise::builtins::autocomplete_command"] command: Option<String>
) -> Result<(), Error> {
    poise::builtins::help(ctx, command.as_deref(), poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
        ..Default::default()
    }).await?;
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "What to vote for"] choice: String
) -> Result<(), Error> {
    let num_votes = {
        let mut hash_map = ctx.data().votes.lock().unwrap();
        let num_votes = hash_map.entry(choice.clone()).or_default();
        *num_votes += 1;
        *num_votes
    };

    let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
pub async fn getvotes(
    ctx: Context<'_>,
    #[description = "Choice to retrieve votes for"] choice: Option<String>
) -> Result<(), Error> {
    if let Some(choice) = choice {
        let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
        let response = match num_votes {
            0 => format!("Nobody has voted for {} yet", choice),
            _ => format!("{} people have voted for {}", num_votes, choice),
        };
        ctx.say(response).await?;
    } else {
        let mut response = String::new();
        for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
            response += &format!("{}: {} votes", choice, num_votes);
        }

        if response.is_empty() {
            response += "Nobody has voted for anything yet :(";
        }

        ctx.say(response).await?;
    }

    Ok(())
}

/// Ping the bot to check its latency
#[poise::command(prefix_command, slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start_time = Instant::now();
    let msg = ctx.say("Pong!").await?;

    let end_time = Instant::now();
    let latency = end_time.duration_since(start_time).as_millis();

    msg.edit(
        ctx,
        poise::CreateReply::default().content(format!("Pong! 🏓 Latency: {}ms", latency))
    ).await?;

    Ok(())
}

#[poise::command(prefix_command)]
pub async fn send_verification(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().id == 829000568559108107 {
        let embed = CreateReply::default().embed(
            CreateEmbed::default()
                .title("Verify")
                .description("React below to get verified")
                .color(0x00ff00)
        );

        let message = ctx.send(embed).await?;

        message
            .into_message().await?
            .react(&ctx.http(), ReactionType::Unicode("✅".to_string())).await?;
    } else {
        ctx.say("You are not authorized to use this command").await?;
    }

    Ok(())
}
