# ron-assista-bot
A bot designed to help RON moderators do things, faster.

## Current Commands
* discordLog - creates a discord infraction log from a id. Requires Bloxlink's global API.
* robloxLog - creates a roblox infraction log from a roblox user.
* probationLog - creates a probation-style infraction log from a discord ID.
* restart - restarts the bot, by killing it and having PM2 restart the entire thing.

### Testing Commands
* attachmentTest - a little trolling, trying to see how I would integrate attachments or get attachments automatically.

# Setup
## Depedencies
* Nodejs: https://nodejs.org/en
* Git: https://git-scm.com
* PM2: https://pm2.io/
* npm: https://www.npmjs.com
* 1 Bloxlink Global API Key.

## Steps
1. Clone this git repository with `git clone https://github.com/RabbyDevs/ron-assista-bot.git`
2. Duplicate `config.example.json` and rename it to `config.json`
3. Add your bot's token, clientID and the bloxlink global API key to said new config file.
4. Delete `config.example.json` if you want.
5. Start the bot with `pm2 ron-assista-bot`
6. Add it to your server(s)!

That's it!
