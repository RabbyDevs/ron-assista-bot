/* eslint-disable max-nested-callbacks */
// Require the necessary discord.js classes
const cacheFilepath = './src/cache.json';
const fs = require('node:fs');
const path = require('node:path');
const { Client, Collection, Events, GatewayIntentBits, REST, Routes } = require('discord.js');
const { token, clientId } = require('./config.json');
const { restart, lastInteractedChannel, user } = require(cacheFilepath);

// Construct and prepare an instance of the REST module
const rest = new REST({ version: '10' }).setToken(token);

// Create a new client instance
const client = new Client({ intents: [
	GatewayIntentBits.Guilds,
	GatewayIntentBits.GuildPresences,
	GatewayIntentBits.GuildMembers,
	GatewayIntentBits.GuildMessages,
	GatewayIntentBits.MessageContent,
	GatewayIntentBits.DirectMessages,
	GatewayIntentBits.DirectMessageTyping,
	GatewayIntentBits.DirectMessageReactions,
] });

client.commands = new Collection();

const commands = [];
// Grab all the command files from the commands directory you created earlier
const foldersPath = path.join(__dirname, 'src/commands');
const commandFolders = fs.readdirSync(foldersPath);

// Grab the SlashCommandBuilder#toJSON() output of each command's data for deployment
for (const folder of commandFolders) {
	const commandsPath = path.join(foldersPath, folder);
	const commandFiles = fs.readdirSync(commandsPath).filter(file => file.endsWith('.js'));
	for (const file of commandFiles) {
		const filePath = path.join(commandsPath, file);
		const command = require(filePath);
		commands.push(command.data.toJSON());
	}
}


// and deploy your commands!
(async () => {
	try {
		console.log(`Started refreshing ${commands.length} application (/) commands.`);

		// The put method is used to fully refresh all commands in the guild with the current set
		const data = await rest.put(
			Routes.applicationCommands(clientId),
			{ body: commands },
		);

		console.log(`Successfully reloaded ${data.length} application (/) commands.`);
	}
	catch (error) {
		// And of course, make sure you catch and log any errors!
		console.error(error);
	}
})();

// change commands for the server.
for (const folder of commandFolders) {
	const commandsPath = path.join(foldersPath, folder);
	const commandFiles = fs.readdirSync(commandsPath).filter(file => file.endsWith('.js'));
	for (const file of commandFiles) {
		const filePath = path.join(commandsPath, file);
		const command = require(filePath);
		// Set a new item in the Collection with the key as the command name and the value as the exported module
		if ('data' in command && 'execute' in command) {
			client.commands.set(command.data.name, command);
		}
		else {
			console.log(`[WARNING] The command at ${filePath} is missing a required "data" or "execute" property.`);
		}
	}
}
// Catch errors from commands.
client.on(Events.InteractionCreate, async interaction => {
	if (!interaction.isChatInputCommand()) return;

	const command = interaction.client.commands.get(interaction.commandName);

	if (!command) {
		console.error(`No command matching ${interaction.commandName} was found.`);
		return;
	}

	try {
		await command.execute(interaction);
	}
	catch (error) {
		console.error(error);
		if (interaction.replied || interaction.deferred) {
			await interaction.followUp({ content: `There was an error while executing this command!\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
		}
		else {
			await interaction.reply({ content: `There was an error while executing this command!\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
		}
	}
});

// When the client is ready, run this code (only once)
// We use 'c' for the event parameter to keep it separate from the already defined 'client'
client.once(Events.ClientReady, c => {
	console.log(`Ready! Logged in as ${c.user.tag}`);
});

// Log in to Discord with your client's token
client.login(token).then(() => {
	// Check if the bot was restarted via /update.
	if (restart == true) {
		client.channels.fetch(lastInteractedChannel)
			.then(channel => {
				console.log('Restart complete!');
				channel.send(`<@${user}> restart complete!`);
			});
		fs.writeFileSync(cacheFilepath, JSON.stringify({ 'restart': false, 'lastInteractedChannel': 0, 'user': 'null' }));
	}
});