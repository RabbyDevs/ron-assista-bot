const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, bloxlinkID, robloxIDtoUser } = require('../../modules/helperFunctions.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('rolelog')
		.setDescription('Makes a log. For role-logging...')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('The ID of the user, separate with spaces.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('role')
				.setDescription('The role your giving, seperate with |')
				.setRequired(true)
				.setChoices(
					{ name: 'Dedicated Player', value: 'Dedicated Player' },
					{ name: 'Collector', value: 'Collector' },
					{ name: 'Grandmaster', value: 'Grandmaster' },
					{ name: 'Gamebanned', value: 'Gamebanned' },
					{ name: 'VIP Blacklist', value: 'VIP Blacklist' },
					{ name: 'Debate Blacklist', value: 'Debate Blacklist' },
					{ name: 'Event Blacklist', value: 'Event Blacklist' },
					{ name: 'Suggestion Blacklist', value: 'Suggestion Blacklist' },
					{ name: 'VC Blackist', value: 'VC Blacklist' },
					{ name: 'Application Blacklist', value: 'Application Blacklist' },
					{ name: 'Creations Blacklist', value: 'Creations Blacklist' },
					{ name: 'Wiki Blacklist', value: 'Wiki Blacklist' },
					{ name: 'Challenges Blacklist', value: 'Challenges Blacklist' },
					{ name: 'Strategies Blacklist', value: 'Strategies Blacklist' },
					{ name: 'Ticket Blacklist', value: 'Ticket Blacklist' },
					{ name: 'Feedback Blacklist', value: 'Feedback Blacklist' },
					{ name: 'Gamebot Blacklist', value: 'Gamebot Blacklist' },
					{ name: 'Trusted VIP Host', value: 'Trusted VIP Host' },
					{ name: 'Ask for Help Blacklist', value: 'Ask for Help Blacklist' },
				))
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		await interaction.deferReply();
		await interaction.editReply('Making log(s), please stand-by!');
		const users = interaction.options.getString('ids').split(' ');
		const role = interaction.options.getString('role').split('|');

		let roleNumber = 0;
		for (const id of users) {
            const robloxId = await bloxlinkID(interaction, id).catch(error => err(interaction, error))
            const robloxUser = await robloxIDtoUser(interaction, await robloxId).catch(error => err(interaction, error))
            await interaction.followUp(`<@${id}>:${id}:${robloxUser}:${robloxId}\n\n${role[roleNumber]}`)
			roleNumber = (role[roleNumber + 1] ? roleNumber + 1 : roleNumber);
		}
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};