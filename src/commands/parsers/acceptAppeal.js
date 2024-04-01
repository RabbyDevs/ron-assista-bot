const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { calculateDuration, getDate } = require('../../modules/helperFunctions');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('acceptappeal')
		.setDescription('Accepts an appeal and sends a message to the user.')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('The ID of the user, separate with spaces.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('duration')
				.setDescription('Duration of probation (only h, d, w, m are usable), make it 0 to have probation not be given.')
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
		await interaction.editReply('Sending accept messages to inputted user(s)...');
		const users = interaction.options.getString('ids').split(' ');
		const duration = interaction.options.getString('duration').split('|');
		const reviewers = (interaction.options.getString('reviewers') ? interaction.options.getString('reviewers').split(' ') : [await interaction.user.id]);

		const calculatedDurations = await calculateDuration(duration);

		let reviewerNumber = 0;
		let durationNumber = 0;
		for (const id of users) {
			// dm the user about their accepted appeal
			const probationString =
            (calculatedDurations[durationNumber] ? `
You are currently on probation for ${calculatedDurations[durationNumber].replace(/.$/, '')}. (<t:${calculatedDurations[durationNumber + 1]}:f> - <t:${calculatedDurations[durationNumber + 2]}:f>)\n
### What is probation?
Probation is a role and if you are warned/muted while your probation has not expired, you will be immediately banned due to being on probation.
On top of this you cannot apply for any Rise of Nations staff positions (including wiki and event ones), and leaving the server will pause your probation time until you have returned.\n\n` : '\n');
			await interaction.client.users.send(id, `Your appeal has been accepted by <@${reviewers[reviewerNumber]}>!\nYou have been unbanned and are now able to rejoin the server.\n${probationString}discord.gg/riseofnations`);
			durationNumber = (calculatedDurations[durationNumber + 3] ? durationNumber + 3 : durationNumber);
			reviewerNumber = (reviewers[reviewerNumber + 1] ? reviewerNumber + 1 : reviewerNumber);
		}
		await interaction.followUp('Done!');
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};