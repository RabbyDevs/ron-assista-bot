const fs = require('fs');
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const filepath = '../../cache.json';
const { getDate } = require('../../modules/helperFunctions.js')
/* A way to update the bot in a simple command. */
module.exports = {
	data: new SlashCommandBuilder()
		.setName('restart')
		.setDescription('Restart the bot, happens automatically if the bot errors!')
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		await interaction.deferReply();
		await interaction.editReply('Restarting bot!');
		let json = {};
		json = { 'restart': true, 'lastInteractedChannel': await interaction.channelId, 'user': await interaction.user.id };
		fs.writeFileSync(filepath, JSON.stringify(json));
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
		process.exit();
	},
};