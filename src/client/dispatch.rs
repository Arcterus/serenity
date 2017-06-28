use std::sync::{Arc, Mutex};
use std::thread;
use super::event_handler::EventHandler;
use super::Context;
use typemap::ShareMap;
use ::gateway::Shard;
use ::model::event::Event;
use ::model::Message;

#[cfg(feature="framework")]
use ::ext::framework::Framework;

#[cfg(feature="cache")]
use super::CACHE;

macro_rules! update {
    ($method:ident, @$event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&mut $event)
            }
        }
    };
    ($method:ident, @$event:expr, $old:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&mut $event, $old)
            }
        }
    };
    ($method:ident, $event:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&$event)
            }
        }
    };
    ($method:ident, $event:expr, $old:expr) => {
        {
            #[cfg(feature="cache")]
            {
                CACHE.write().unwrap().$method(&$event, $old)
            }
        }
    };
}

fn context(conn: &Arc<Mutex<Shard>>,
           data: &Arc<Mutex<ShareMap>>) -> Context {
    Context::new(conn.clone(), data.clone())
}

#[cfg(feature="framework")]
pub fn dispatch<H: EventHandler + Send + Sync + 'static>(event: Event,
                conn: &Arc<Mutex<Shard>>,
                framework: &Arc<Mutex<Framework>>,
                data: &Arc<Mutex<ShareMap>>,
                event_handler: &Arc<H>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data);
            let mut framework = framework.lock().unwrap();

            if framework.initialized {
                dispatch_message(context.clone(),
                                 event.message.clone(),
                                 event_handler);

                framework.dispatch(context, event.message);
            } else {
                dispatch_message(context, event.message, event_handler);
            }
        },
        other => handle_event(other, conn, data, event_handler),
    }
}

#[cfg(not(feature="framework"))]
pub fn dispatch<H: EventHandler + Send + Sync + 'static>(event: Event,
                conn: &Arc<Mutex<Shard>>,
                data: &Arc<Mutex<ShareMap>>,
                event_handler: &Arc<H>) {
    match event {
        Event::MessageCreate(event) => {
            let context = context(conn, data);
            dispatch_message(context,
                             event.message,
                             event_handler);
        },
        other => handle_event(other, conn, data, event_handler),
    }
}

#[allow(unused_mut)]
fn dispatch_message<H: EventHandler + Send + Sync + 'static>(context: Context,
                    mut message: Message,
                    event_handler: &Arc<H>) {
    let h = event_handler.clone();
    thread::spawn(move || {
        #[cfg(feature="model")]
        {
            message.transform_content();
        }

        h.on_message(context, message);
    });
}

#[allow(cyclomatic_complexity, unused_assignments, unused_mut)]
fn handle_event<H: EventHandler + Send + Sync + 'static>(event: Event,
                conn: &Arc<Mutex<Shard>>,
                data: &Arc<Mutex<ShareMap>>,
                event_handler: &Arc<H>) {
    match event {
        Event::ChannelCreate(event) => {
            update!(update_with_channel_create, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_channel_create(context, event.channel));
        },
        Event::ChannelDelete(event) => {
            update!(update_with_channel_delete, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_channel_delete(context, event.channel));
        },
        Event::ChannelPinsUpdate(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_channel_pins_update(context, event));
        },
        Event::ChannelRecipientAdd(mut event) => {
            update!(update_with_channel_recipient_add, @event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_channel_recipient_addition(context, event.channel_id, event.user));
        },
        Event::ChannelRecipientRemove(event) => {
            update!(update_with_channel_recipient_remove, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_channel_recipient_removal(context, event.channel_id, event.user));
        },
        Event::ChannelUpdate(event) => {
            update!(update_with_channel_update, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                let before = CACHE.read().unwrap().channel(event.channel.id());
                thread::spawn(move || h.on_channel_update(context, before, event.channel));
            } else {
                thread::spawn(move || h.on_channel_update(context, event.channel));
            }}
        },
        Event::GuildBanAdd(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_ban_addition(context, event.guild_id, event.user));
        },
        Event::GuildBanRemove(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_ban_removal(context, event.guild_id, event.user));
        },
        Event::GuildCreate(event) => {
            update!(update_with_guild_create, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_create(context, event.guild));
        },
        Event::GuildDelete(event) => {
            let _full = update!(update_with_guild_delete, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                thread::spawn(move || h.on_guild_delete(context, event.guild, _full));
            } else {
                thread::spawn(move || h.on_guild_delete(context, event.guild));
            }}
        },
        Event::GuildEmojisUpdate(event) => {
            update!(update_with_guild_emojis_update, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_emojis_update(context, event.guild_id, event.emojis));
        },
        Event::GuildIntegrationsUpdate(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_integrations_update(context, event.guild_id));
        },
        Event::GuildMemberAdd(mut event) => {
            update!(update_with_guild_member_add, @event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_member_addition(context, event.guild_id, event.member));
        },
        Event::GuildMemberRemove(event) => {
            let _member = update!(update_with_guild_member_remove, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                thread::spawn(move || h.on_guild_member_removal(context, event.guild_id, event.user, _member));
            } else {
                thread::spawn(move || h.on_guild_member_removal(context, event.guild_id, event.user));
            }}
        },
        Event::GuildMemberUpdate(event) => {
            let _before = update!(update_with_guild_member_update, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                // This is safe to unwrap, as the update would have created
                // the member if it did not exist. So, there is be _no_ way
                // that this could fail under any circumstance.
                let after = CACHE.read()
                    .unwrap()
                    .member(event.guild_id, event.user.id)
                    .unwrap()
                    .clone();

                thread::spawn(move || h.on_guild_member_update(context, _before, after));
            } else {
                thread::spawn(move || h.on_guild_member_update(context, event));
            }}
        },
        Event::GuildMembersChunk(event) => {
            update!(update_with_guild_members_chunk, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_members_chunk(context, event.guild_id, event.members));
        },
        Event::GuildRoleCreate(event) => {
            update!(update_with_guild_role_create, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_role_create(context, event.guild_id, event.role));
        },
        Event::GuildRoleDelete(event) => {
            let _role = update!(update_with_guild_role_delete, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                thread::spawn(move || h.on_guild_role_delete(context, event.guild_id, event.role_id, _role));
            } else {
                thread::spawn(move || (handler)(context, event.guild_id, event.role_id));
            }}
        },
        Event::GuildRoleUpdate(event) => {
            let _before = update!(update_with_guild_role_update, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                thread::spawn(move || h.on_guild_role_update(context, event.guild_id, _before, event.role));
            } else {
                thread::spawn(move || h.on_guild_role_update(context, event.guild_id, event.role));
            }}
        },
        Event::GuildUnavailable(event) => {
            update!(update_with_guild_unavailable, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_guild_unavailable(context, event.guild_id));
        },
        Event::GuildUpdate(event) => {
            update!(update_with_guild_update, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                let before = CACHE.read()
                    .unwrap()
                    .guilds
                    .get(&event.guild.id)
                    .cloned();
        
                thread::spawn(move || h.on_guild_update(context, before, event.guild));
            } else {
                thread::spawn(move || h.on_guild_update(context, event.guild));
            }}
        },
        // Already handled by the framework check macro
        Event::MessageCreate(_) => {},
        Event::MessageDeleteBulk(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_message_delete_bulk(context, event.channel_id, event.ids));
        },
        Event::MessageDelete(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_message_delete(context, event.channel_id, event.message_id));
        },
        Event::MessageUpdate(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_message_update(context, event));
        },
        Event::PresencesReplace(event) => {
            update!(update_with_presences_replace, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_presence_replace(context, event.presences));
        },
        Event::PresenceUpdate(mut event) => {
            update!(update_with_presence_update, @event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_presence_update(context, event));
        },
        Event::ReactionAdd(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_reaction_add(context, event.reaction));
        },
        Event::ReactionRemove(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_reaction_remove(context, event.reaction));
        },
        Event::ReactionRemoveAll(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_reaction_remove_all(context, event.channel_id, event.message_id));
        },
        Event::Ready(event) => {
            update!(update_with_ready, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_ready(context, event.ready));
        },
        Event::Resumed(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_resume(context, event));
        },
        Event::TypingStart(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_typing_start(context, event));
        },
        Event::Unknown(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_unknown(context, event.kind, event.value));
        },
        Event::UserUpdate(event) => {
            let _before = update!(update_with_user_update, event);
            let context = context(conn, data);

            let h = event_handler.clone();
            feature_cache! {{
                thread::spawn(move || h.on_user_update(context, _before, event.current_user));
            } else {
                thread::spawn(move || h.on_user_update(context, event.current_user));
            }}
        },
        Event::VoiceServerUpdate(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_voice_server_update(context, event));
        },
        Event::VoiceStateUpdate(event) => {
            update!(update_with_voice_state_update, event);

            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_voice_state_update(context, event.guild_id, event.voice_state));
        },
        Event::WebhookUpdate(event) => {
            let context = context(conn, data);

            let h = event_handler.clone();
            thread::spawn(move || h.on_webhook_update(context, event.guild_id, event.channel_id));
        },
    }
}
