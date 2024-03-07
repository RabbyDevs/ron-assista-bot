/* eslint-disable no-unused-vars */
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { getDate, robloxUsertoID, robloxInfoFromID, bloxlinkID, badgeInfoFromID, robloxFriendCountFromID, robloxGroupCountFromID } = require('../../modules/helperFunctions');

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
		.addIntegerOption(option =>
			option
				.setName('badge-max-iterations')
				.setDescription('Maximum number of iterations to do when doing badge info, maximum of 10.')
				.setRequired(false)
				.setMaxValue(10))
		.addBooleanOption(option =>
			option
				.setName('badge-info')
				.setDescription('Should the bot retrieve badge info?')
				.setRequired(false))
		// .addIntegerOption(option =>
		// 	option
		// 		.setName('inventory-max-iterations')
		// 		.setDescription('Maximum number of iterations to do when doing inventory info, maximum of 10.')
		// 		.setRequired(false)
		// 		.setMaxValue(10))
		// .addBooleanOption(option =>
		// 	option
		// 		.setName('inventory-info')
		// 		.setDescription('Should the bot retrieve inventory info?')
		// 		.setRequired(false))
		
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Finding information and outputting, standby!');
		console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		// variables/arguments
		const roblox_ids = (interaction.options.getString('roblox-ids') ? interaction.options.getString('roblox-ids').split(' ') : []);
		const roblox_users = (interaction.options.getString('roblox-users') ? interaction.options.getString('roblox-users').split(' ') : []);
        const discord_ids = (interaction.options.getString('discord-ids') ? interaction.options.getString('discord-ids').split(' ') : []);
		const badge_max_iterations = (interaction.options.getInteger('badge-max-iterations') ? interaction.options.getInteger('badge-max-iterations') : 1)
		const badges_enabled = (interaction.options.getBoolean('badge-info') ? interaction.options.getBoolean('badge-info') : false)
		for (const user of roblox_users) {roblox_ids.push(await robloxUsertoID(interaction, [user]))}
        for (const id of discord_ids) {await roblox_ids.push(await bloxlinkID(interaction, id))}
		if (roblox_ids[0] == undefined) {
			interaction.followUp(`<@${interaction.user.id}> command failed! No ids inputted.`) 
			console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
			return
		}
        for (const id of roblox_ids) {
            const information = await robloxInfoFromID(interaction, id)
            await interaction.channel.send('\\- Username -')
            await interaction.channel.send(information.name)
            await interaction.channel.send('\\- User ID -')
            await interaction.channel.send(`${information.id}`)
            await interaction.channel.send(`
\\- Profile Link -
https://roblox.com/users/${id}
\\- Description -
${(information.description ? information.description : 'No description.')}
\\- Account Creation Date -
<t:${Math.round(Date.parse(information.created)/1000)}:D>
\\- Friend Count -
${await robloxFriendCountFromID(interaction, id)}
\\- Group Count -
${await robloxGroupCountFromID(interaction, id)}`)
			if (badges_enabled == true) {
				const msg = await interaction.channel.send({ content: 'Getting badge info, standby this may take a **while**!'})
				const badgeInfo = await badgeInfoFromID(interaction, id, badge_max_iterations)
				await interaction.channel.send(`
\\- BADGE INFO -
Badge Count: ${badgeInfo[0]}
Average Win-rate: ${badgeInfo[1]}
Welcome Badge Count: ${badgeInfo[2]}
Top 5 Places for User:
${(badgeInfo[3][0] ? `- ${badgeInfo[3][0][0]}: ${badgeInfo[3][0][1]}` : ``)}
${(badgeInfo[3][1] ? `- ${badgeInfo[3][1][0]}: ${badgeInfo[3][1][1]}` : ``)}
${(badgeInfo[3][2] ? `- ${badgeInfo[3][2][0]}: ${badgeInfo[3][2][1]}` : ``)}
${(badgeInfo[3][3] ? `- ${badgeInfo[3][3][0]}: ${badgeInfo[3][3][1]}` : ``)}
${(badgeInfo[3][4] ? `- ${badgeInfo[3][4][0]}: ${badgeInfo[3][4][1]}` : ``)}
`)
				msg.delete()
			}
        }
		console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};