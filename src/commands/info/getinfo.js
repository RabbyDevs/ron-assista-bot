/* eslint-disable no-unused-vars */
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, robloxUsertoID, robloxInfoFromID, bloxlinkID, delay } = require('../../modules/helperFunctions');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('getinfo')
		.setDescription('Obtains info from a Roblox user.')
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
				.setName('discord-ids')
				.setDescription('Discord ID(s) to make log from, use a space to separate users.')
				.setRequired(false))
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Finding information and outputting, standby!');
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		// variables/arguments
		const roblox_ids = (interaction.options.getString('roblox-ids') ? interaction.options.getString('roblox-ids').split(' ') : []);
		const roblox_users = (interaction.options.getString('roblox-users') ? interaction.options.getString('roblox-users').split(' ') : []);
        const discord_ids = (interaction.options.getString('discord-ids') ? interaction.options.getString('discord-ids').split(' ') : []);
		for (const user of roblox_users) {roblox_ids.push(await robloxUsertoID(interaction, [user]))}
        for (const id of discord_ids) {await roblox_ids.push(await bloxlinkID(interaction, id))}
        for (const id of roblox_ids) {
            console.log(id)
            const information = await robloxInfoFromID(interaction, id)
            interaction.channel.send('\\- Username -')
            interaction.channel.send(information.name)
            interaction.channel.send('\\- User ID -')
            interaction.channel.send(`${information.id}`)
            interaction.channel.send('\\- Profile Link -')
            interaction.channel.send(`https://roblox.com/users/${id}`)
            interaction.channel.send('\\- Description -')
            interaction.channel.send(information.description)
            interaction.channel.send('\\- Account Creation Date -')
            interaction.channel.send(`<t:${Math.round(Date.parse(information.created))}:D>`)
        }
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};