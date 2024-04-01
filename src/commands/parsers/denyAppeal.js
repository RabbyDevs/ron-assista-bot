const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { getDate } = require('../../modules/helperFunctions')

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
		.addStringOption(option =>
			option
				.setName('reviewers')
				.setDescription('IDs of the people who reviewed appeal(s). Separate with spaces.')
				.setRequired(false))
		.setDefaultMemberPermissions(PermissionFlagsBits.Administrator),
	async execute(interaction) {
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		await interaction.deferReply();
		await interaction.editReply('Sending reject messages to inputted user(s)...');
		const users = interaction.options.getString('ids').split(' ');
		const reason = interaction.options.getString('reason').split('|');
		const reviewers = (interaction.options.getString('reviewers') ? interaction.options.getString('reviewers').split(' ') : [await interaction.user.id]);

		let reasonNumber = 0;
		let reviewerNumber = 0;
		for (const id of users) {
			await interaction.client.users.send(id, `Your appeal has been denied by <@${reviewers[reviewerNumber]}>.\nReason - ${reason[reasonNumber]}`);
			reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
			reviewerNumber = (reviewers[reviewerNumber + 1] ? reviewerNumber + 1 : reviewerNumber);
		}
		await interaction.followUp('Done!');
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};