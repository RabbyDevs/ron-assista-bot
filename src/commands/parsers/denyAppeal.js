const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('denyappeal')
		.setDescription('Denies an appeal and sends a message to the user.')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('The ID of the user, separate with spaces.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('reason')
				.setDescription('Reason of denial, separate with "|".')
				.setRequired(true))
		.setDefaultMemberPermissions(PermissionFlagsBits.KickMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Sending reject messages to inputted user(s)...');
		const users = interaction.options.getString('ids').split(' ');
		const reason = interaction.options.getString('reason').split('|');

		let reasonNumber = 0;
		for (const id of users) {
			await interaction.client.users.send(id, `Your appeal has been denied by <@${await interaction.user.id}>.\nReason - ${reason[reasonNumber]}`);
			reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
		}
		await interaction.followUp('Done!');
	},
};