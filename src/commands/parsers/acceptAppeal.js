const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');

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
		await interaction.deferReply();
		await interaction.editReply('Sending accept messages to inputted user(s)...');
		const users = interaction.options.getString('ids').split(' ');
		const duration = interaction.options.getString('duration').split('|');
		const reviewers = (interaction.options.getString('reviewers') ? interaction.options.getString('reviewers').split(' ') : [await interaction.user.id]);

		const calculatedDurations = [];
		let durNumber = 0;
		for (const dateID in duration) {
			// epoch part
			const dates = { 'h': 3600, 'd': 86400, 'w': 604800, 'm': 2629743 };
			const date = (duration[dateID].replace(/ /gi, '') == 0 ? '0h' : duration[dateID].replace(/ /gi, ''));
			const numbers = date.split(/[dwmh]/gi);
			numbers.pop();
			const letters = date.split(/[1234567890]/gi);
			letters.shift();

			const today = Date.now();
			const todayEpoch = Math.floor(today / 1000);
			let epoch = Math.floor(today / 1000);
			for (const number in numbers) {
				const combinedNumber = numbers[number] * dates[letters[number]];
				epoch = epoch + combinedNumber;
			}

			// full duration part
			const fullMonths = { 'h': 'Hour', 'd': 'Day', 'w': 'Week', 'm': 'Month' };
			let fullDuration = '';
			for (const letter in letters) {
				let month = (numbers[letter] > 1 ? `${fullMonths[letters[letter]]}s` : `${fullMonths[letters[letter]]}`);
				month += (letter == letters.length - 2 || letter == letters.length - 1 ? (letter == letters.length - 2 ? ' and' : '') : ',');
				fullDuration += `${numbers[letter]} ${month} `;
			}
			calculatedDurations[durNumber + 1] = (date !== '0' ? todayEpoch : undefined);
			calculatedDurations[durNumber + 2] = (date !== '0' ? epoch : undefined);
			calculatedDurations[durNumber] = (date !== '0' ? fullDuration : undefined);
			durNumber = durNumber + 3;
		}

		let reviewerNumber = 0;
		let durationNumber = 0;
		for (const id of users) {
			// dm the user about their accepted appeal
			const probationString =
            (calculatedDurations[durationNumber] ? `
You are currently on probation for ${calculatedDurations[durationNumber].replace(/.$/, '')}. (<t:${calculatedDurations[durationNumber + 1]}:f> - <t:${calculatedDurations[durationNumber + 2]}:f>)\nNotify a staff member that you are on probation so you can receive the role!\n
You cannot apply for any official Rise of Nations position while on probation.
If a warn or mute is given during probation, you will be immediately banned due to being on probation.
Leaving the server during your Probation will pause it until you have returned.\n` : '');
			await interaction.client.users.send(id, `Your appeal has been accepted by <@${reviewers[reviewerNumber]}>!\nYou have been unbanned and are now able to rejoin the server.\n${probationString}\ndiscord.gg/riseofnations`);
			durationNumber = (calculatedDurations[durationNumber + 3] ? durationNumber + 3 : durationNumber);
			reviewerNumber = (reviewers[reviewerNumber + 1] ? reviewerNumber + 1 : reviewerNumber);
		}
		await interaction.followUp('Done!');
	},
};