const fs = require('fs');
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
let json = {};
const filepath = '../../cache.json';
/* A way to update the bot in a simple command. */
module.exports = {
	data: new SlashCommandBuilder()
		.setName('restart')
		.setDescription('Restart the bot, happens automatically if the bot errors!')
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		console.log(`Bot restart initiated by ${interaction.user.username}!`);
		await interaction.deferReply();
		await interaction.editReply('Restarting bot!');
		json = { 'restart': true, 'lastInteractedChannel': await interaction.channelId, 'user': await interaction.user.id };
		fs.writeFileSync(filepath, JSON.stringify(json));
		process.exit();
	},
};