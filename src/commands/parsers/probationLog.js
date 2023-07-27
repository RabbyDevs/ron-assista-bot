const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, robloxUsertoID, bloxlinkID, calculateDuration, robloxIDtoUser } = require('/home/rabby/ron-assista-bot/src/modules/helperFunctions.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('probationlog')
		.setDescription('Replies with the probation log given a Discord ID or multiple.')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('ID(s) to make log from, use a space to separate users.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('reason')
				.setDescription('Reason for log can be split via |.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('duration')
				.setDescription('Duration of probation (only h, d, w, m are usable).')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('timeformat')
				.setDescription('Time format of probation (look up a guide on google, defaults to f).')
				.setRequired(false))
		.setDefaultMemberPermissions(PermissionFlagsBits.Administrator),
	async execute(interaction) {
		await interaction.deferReply();
		// detect if the user is on mobile on any platform:
		await interaction.editReply('Making log(s), please stand-by!');
		console.log(`Command getdiscordlog begun on ${await getDate()[0]} by ${interaction.user.username}, with parameters: ${interaction.options.getString('ids')}, ${interaction.options.getString('type')}, ${interaction.options.getString('reason')}, ${interaction.options.getString('note')}, ${interaction.options.getBoolean('multimessage')}.`);
		// variables/arguments
		const users = interaction.options.getString('ids').split(' ');
		const reason = interaction.options.getString('reason').split('|');
		const duration = interaction.options.getString('duration').split('|');
		const timeFormat = (interaction.options.getString('timeFormat') ? interaction.options.getString('timeFormat').split(' ') : 'f');
		// possibly split at a space, then parse by checking the ending letter against a dictionary of items?
		// okk now checking how to check the last letter of a string :3
		const calculatedDurations = await calculateDuration(duration);

		// make a multiple msg log from arguments + table magic
		async function multiLog() {
			let reasonNumber = 0;
			let durationNumber = 0;
			for (const id of users) {
				// make a log
				const robloxId = await bloxlinkID(id).catch(error => err(interaction, error));
				const robloxUser = await robloxIDtoUser(await robloxId).catch(error => err(interaction, error));
				let text = '';
				text += `[<@${id}> - ${id} - ${robloxUser}:${robloxId}]\n\n`;
				text += `[${reason[reasonNumber]}]\n\n`;
				text += `[${calculatedDurations[durationNumber]}(<t:${calculatedDurations[durationNumber + 1]}:${timeFormat}> - <t:${calculatedDurations[durationNumber + 2]}:${timeFormat}>)]`;
				await interaction.followUp(text);
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				durationNumber = (calculatedDurations[durationNumber + 3] ? durationNumber + 3 : durationNumber);
			}
		}
		// command logic
		multiLog();
		console.log(`Command getdiscordlog started by ${interaction.user.username} ended on ${await getDate()[0]}`);
	},
};