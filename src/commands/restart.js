const fs = require('fs');
const { SlashCommandBuilder } = require('discord.js');
let json = {};
const filepath = '/home/rabby/ron-assista-bot/src/cache.json';
/* A way to update the bot in a simple command. */
module.exports = {
	data: new SlashCommandBuilder()
		.setName('restart')
		.setDescription('Restart the bot, happens automatically if the bot errors!'),
	async execute(interaction) {
		console.log('Bot restart initiated!');
		await interaction.deferReply();
		await interaction.editReply('Restarting bot!');
		json = { 'restart': true, 'lastInteractedChannel': await interaction.channelId, 'user': await interaction.user.id };
		fs.writeFileSync(filepath, JSON.stringify(json));
		process.exit();
	},
};