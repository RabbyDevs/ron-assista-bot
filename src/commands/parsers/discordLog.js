/* eslint-disable no-unused-vars */
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const { err, getDate, robloxUsertoID, bloxlinkID, robloxIDtoUser } = require('../../modules/helperFunctions.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('discordlog')
		.setDescription('Replies with a proper RON Log when given Discord User.')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('ID(s) to make log from, use a space to separate users.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('type')
				.setDescription('Type of infraction')
				.setRequired(true)
				.setChoices(
					{ name: 'Discord: Ban', value: 'Ban' },
					{ name: 'Discord: Temporary Ban', value: 'Temporary Ban' },
					{ name: 'Discord: Kick', value: 'Kick' },
					{ name: 'Discord: Mute', value: 'Mute' },
					{ name: 'Discord: Warn', value: 'Warn' },
				))
		.addStringOption(option =>
			option
				.setName('reason')
				.setDescription('Reason for log can be split via |, split only works if multimessage is True.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('duration')
				.setDescription('Duration of the mute, or Temporary Ban.')
				.setRequired(false))
		.addStringOption(option =>
			option
				.setName('note')
				.setDescription('Extra notes, can be split via |, split only works if multimessage is True.')
				.setRequired(false))
		.addBooleanOption(option =>
			option
				.setName('multimessage')
				.setDescription('Should the bot split logs into multiple messages if there are multiple users?')
				.setRequired(false))
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Making log(s), please stand-by!');
		console.log(`Command getdiscordlog begun on ${await getDate()} by ${interaction.user.username}, with parameters: ${interaction.options.getString('ids')}, ${interaction.options.getString('type')}, ${interaction.options.getString('reason')}, ${interaction.options.getString('note')}, ${interaction.options.getBoolean('multimessage')}.`);
		// variables/arguments
		const users = interaction.options.getString('ids').split(' ');
		const type = interaction.options.getString('type');
		const reason = interaction.options.getString('reason').split('|');
		const notes = (interaction.options.getString('note') ? interaction.options.getString('note').split('|') : [undefined]);
		const multiMessage = (interaction.options.getBoolean('multimessage') ? interaction.options.getBoolean('multimessage') : false);
		const duration = (type == 'Mute' || type == 'Temporary Ban' ? interaction.options.getString('duration') : undefined);
		// make a single log from above arguments.
		async function singleLog() {
			let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
			for (const id of users) {
				const robloxId = (type == 'Ban' ? await bloxlinkID(interaction, id).catch(error => err(interaction, error)) : undefined);
				const robloxUser = (type == 'Ban' ? await robloxIDtoUser(interaction, await robloxId).catch(error => err(interaction, error)) : undefined);
				text += (robloxId ? `[<@${id}>:${id}:${robloxUser}:${robloxId}]\n` : `[<@${id}>:${id}]\n`);
			}
			text += (notes[0] ? `[${reason[0]}]\nNote: ${notes[0]}` : `[${reason[0]}]`);
			await interaction.followUp(text);
		}
		// make a multiple msg log from arguments + table magic
		async function multiLog() {
			let reasonNumber = 0;
			let noteNumber = 0;
			for (const id of users) {
				const robloxId = (type == 'Ban' ? await bloxlinkID(interaction, id).catch(error => err(interaction, error)) : undefined);
				const robloxUser = (type == 'Ban' ? await robloxIDtoUser(interaction, await robloxId).catch(error => err(interaction, error)) : undefined);
				let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
				text += (robloxId ? `[<@${id}>:${id}:${robloxUser}:${robloxId}]\n` : `[<@${id}>:${id}]\n`);
				text += `[${reason[reasonNumber]}]`;
				text += (notes[noteNumber] ? `\nNote: ${notes[noteNumber]}` : '');
				await interaction.followUp(text);
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				noteNumber = (notes[noteNumber + 1] ? noteNumber + 1 : noteNumber);
			}
		}
		// command logic
		(multiMessage == true ? multiLog() : singleLog());
		console.log(`Command getdiscordlog started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};