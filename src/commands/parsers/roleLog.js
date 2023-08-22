const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, robloxUsertoID, bloxlinkID, calculateDuration, robloxIDtoUser } = require('../../modules/helperFunctions.js');

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
				.setRequired(true))
		.setDefaultMemberPermissions(PermissionFlagsBits.KickMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Making log(s), please stand-by!');
		const users = interaction.options.getString('ids').split(' ');
		const role = interaction.options.getString('role').split('|');

		let roleNumber = 0;
		for (const id of users) {
            const robloxId = await bloxlinkID(id).catch(error => err(interaction, error))
            const robloxUser = await robloxIDtoUser(await robloxId).catch(error => err(interaction, error))
            await interaction.followUp(`<@${id}>:${id}:${robloxUser}:${robloxId}\n\n${role[roleNumber]}`)
			roleNumber = (role[roleNumber + 1] ? roleNumber + 1 : roleNumber);
		}
	},
};