const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, robloxUsertoID, bloxlinkID, calculateDuration, robloxIDtoUser } = require('../../modules/helperFunctions.js');

module.exports = {
	data: new SlashCommandBuilder()
	.setName('probationlog')
	.setDescription('Replies with the probation log given a Discord ID or multiple.')
	.addStringOption(option =>
		option
			.setName('type')
			.setDescription('Type of infraction.')
			.setRequired(true)
			.setChoices(
				{ name: 'Roblox', value: 'Roblox' },
				{ name: 'Discord', value: 'Discord' },
			))
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
			.setName('discord-ids')
			.setDescription('Discord ID(s) to make log from, use a space to separate users.')
			.setRequired(false))
	.addStringOption(option =>
		option
			.setName('roblox-ids')
			.setDescription('Roblox ID(s) to make log from, use a space to separate users.')
			.setRequired(false))
	.addStringOption(option =>
		option
			.setName('roblox-users')
			.setDescription('Roblox usernames(s) to make log from, use a space to separate users.')
			.setRequired(false))
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
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		// variables/arguments
		const type = interaction.options.getString('type');
		const discord_ids = (interaction.options.getString('discord-ids') ? interaction.options.getString('discord-ids').split(' ') : []);
		const roblox_ids = (interaction.options.getString('roblox-ids') ? interaction.options.getString('roblox-ids').split(' ') : []);
		const roblox_users = (interaction.options.getString('roblox-users') ? interaction.options.getString('roblox-users').split(' ') : []);
		for (const id of roblox_users) {roblox_ids.push(await robloxUsertoID(interaction, [id]))}
		const reason = interaction.options.getString('reason').split('|');
		const duration = interaction.options.getString('duration').split('|');
		const timeFormat = (interaction.options.getString('timeFormat') ? interaction.options.getString('timeFormat').split(' ') : 'D');
		// possibly split at a space, then parse by checking the ending letter against a dictionary of items?
		// okk now checking how to check the last letter of a string :3
		const calculatedDurations = await calculateDuration(duration);

		const all_ids = discord_ids.concat(roblox_ids)

		// make a multiple msg log from arguments + table magic
		async function multiLog() {
			let reasonNumber = 0;
			let durationNumber = 0;
			for (const flake of all_ids) {
				// make a log
				const regExp = /[a-zA-Z]/g;
				let firstID
				let firstUser
				let secondID
				let secondUser
				if(flake.length > 12){
					firstID = flake
					firstUser = `<@${flake}>`
					secondID = await bloxlinkID(interaction, flake).catch(error => err(interaction, error));
					secondUser = await robloxIDtoUser(interaction, await secondID).catch(error => err(interaction, error));
				} else {
					firstUser = await robloxIDtoUser(interaction, flake)
					firstID = flake
				}
				let text = '';
				text += `[${type} Ban]\n\n`
				text += `[${firstUser}:${firstID}`
				console.log(firstID, firstUser)
				secondUser ? text += ` - ${secondUser}:${secondID}]\n\n` : text += `]\n\n`
				text += `[${reason[reasonNumber]}]\n\n`;
				text += `[${calculatedDurations[durationNumber]}(<t:${calculatedDurations[durationNumber + 1]}:${timeFormat}> - <t:${calculatedDurations[durationNumber + 2]}:${timeFormat}>)]`;
				await interaction.followUp(text);
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				durationNumber = (calculatedDurations[durationNumber + 3] ? durationNumber + 3 : durationNumber);
			}
		}
		// command logic
		multiLog();
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};