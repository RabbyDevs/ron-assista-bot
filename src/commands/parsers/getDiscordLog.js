/* eslint-disable no-unused-vars */
const { SlashCommandBuilder } = require('discord.js');
const http = require('https');

const { bloxlinkAPIKey } = require('/home/rabby/ron-assista-bot/config.json');

// helper function: error the command
async function err(interaction, error) {
	await interaction.editReply({ content: `**There was an error while executing this command!**\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
	throw error;
}

// helper function: get the current date.
async function getDate() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
}

// Gets the RobloxID of a discord user VIA bloxlink global api.
async function getRobloxId(userId) {
	const id = new Promise((resolve, reject) => {
		http.get(`https://api.blox.link/v4/public/discord-to-roblox/${userId}`, {
			headers: { 'Authorization': bloxlinkAPIKey },
		}, (response) => {
			const data = [];
			const headerDate = response.headers && response.headers.date ? response.headers.date : 'no response date';
			console.log(`Bloxlink GET request started by getRobloxId. Input is: ${userId}`);
			console.log('Status Code:', response.statusCode);
			console.log('Date in Response header:', headerDate);

			response.on('data', chunk => {
				data.push(chunk);
			});

			response.on('end', () => {
				console.log(`Response ended in: ${data}`);
				const obj = JSON.parse(data);
				if (response.statusCode == 200) {
					resolve(obj.robloxID);
				}
				else {
					reject(obj.error);
				}
			});
		});
	});
	return id;
}

// Gets the username from a Roblox ID via Roblox official API endpoints.
async function getUserFromRobloxId(robloxId) {
	const username = new Promise((resolve, reject) => {
		http.get(`https://users.roblox.com/v1/users/${robloxId}`, (response) => {
			const data = [];
			const headerDate = response.headers && response.headers.date ? response.headers.date : 'no response date';
			console.log('Roblox GET request started by getUserFromRobloxId!');
			console.log('Status Code:', response.statusCode);
			console.log('Date in Response header:', headerDate);

			response.on('data', chunk => {
				data.push(chunk);
			});

			response.on('end', () => {
				console.log(`Response ended in: ${data}`);
				const obj = JSON.parse(data);
				if (obj.errors == undefined) {
					resolve(obj.name);
				}
				else {
					reject(obj.errors.message);
				}
			});
		});
	});
	return username;
}

module.exports = {
	data: new SlashCommandBuilder()
		.setName('getdiscordlog')
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
				.setName('note')
				.setDescription('Extra notes, can be split via |, split only works if multimessage is True.')
				.setRequired(false))
		.addBooleanOption(option =>
			option
				.setName('multimessage')
				.setDescription('Should the bot split logs into multiple messages if there are multiple users?')
				.setRequired(false))
		.addStringOption(option =>
			option
				.setName('duration')
				.setDescription('Duration of the mute, or Temporary Ban.')
				.setRequired(false),
		),
	async execute(interaction) {
		await interaction.deferReply();
		// detect if the user is on mobile on any platform:
		const isMobile = (await interaction.member.presence.clientStatus.mobile ? true : false);
		(isMobile == true ? await interaction.editReply('Mobile detected! Adding mobile friendly log.') : await interaction.editReply('Making log(s), please stand-by!'));
		console.log(`Command getdiscordlog begun on ${await getDate()} by ${interaction.user.username}, with parameters: ${interaction.options.getString('ids')}, ${interaction.options.getString('type')}, ${interaction.options.getString('reason')}, ${interaction.options.getString('note')}, ${interaction.options.getBoolean('multimessage')}.`);
		// variables/arguments
		const users = interaction.options.getString('ids').split(' ');
		const type = interaction.options.getString('type');
		const reason = interaction.options.getString('reason').split('|');
		const note = (interaction.options.getString('note') ? interaction.options.getString('note').split('|') : { undefined });
		const multiMessage = (interaction.options.getBoolean('multimessage') ? interaction.options.getBoolean('multimessage') : false);
		const duration = (type == 'Mute' || type == 'Temporary Ban' ? interaction.options.getString('duration') : undefined);
		// make a single log from above arguments.
		async function makeSingleLog() {
			let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
			for (const id of users) {
				if (type == 'Ban') {
					const robloxId = await getRobloxId(id).catch(error => err(interaction, error));
					const robloxUser = await getUserFromRobloxId(await robloxId).catch(error => err(interaction, error));
					text += (isMobile == true ? `[<@${id}>:${id}:${robloxUser}:${robloxId}]` : `[<\\@${id}>:${id}:${robloxUser}:${robloxId}]`);
				}
				else {
					text += (isMobile == true ? `[<@${id}>:${id}]` : `[<\\@${id}>:${id}]`);
				}
			}
			text += (note[0] ? `\n[${reason[0]}]\nNote: ${note[0]}` : `\n[${reason[0]}]`);
			await interaction.followUp(text);
			(isMobile == true ? await interaction.followUp('Desktop version of the log:\n' + text.replace('<@', '<\\@')) : undefined);
		}
		// make a multiple msg log from arguments + table magic
		async function multiLog() {
			let reasonNumber = 0;
			let noteNumber = 0;
			for (const id of users) {
				let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
				if (type == 'Ban') {
					const robloxId = await getRobloxId(id).catch(error => err(interaction, error));
					const robloxUser = await getUserFromRobloxId(await robloxId.catch(error => err(interaction, error)));
					text += (isMobile == true ? `[<@${id}>:${id}:${robloxUser}:${robloxId}]` : `[<\\@${id}>:${id}:${robloxUser}:${robloxId}]`);
					text += `\n[${reason[reasonNumber]}]`;
					text += (note[noteNumber] ? `\nNote: ${note[noteNumber]}` : '');
					await interaction.followUp(text);
					(isMobile == true ? await interaction.followUp('Desktop version of the log:\n' + text.replace('<@', '<\\@')) : undefined);
				}
				else {
					text += (isMobile == true ? `[<@${id}>:${id}]` : `[<\\@${id}>:${id}]`);
					text += `\n[${reason[reasonNumber]}]`;
					text += (note[noteNumber] ? `\nNote: ${note[noteNumber]}` : '');
					await interaction.followUp(text);
					(isMobile == true ? await interaction.followUp('Desktop version of the log:\n' + text.replace('<@', '<\\@')) : undefined);
				}
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				note[noteNumber] = null;
				noteNumber = (note[noteNumber + 1] ? noteNumber + 1 : noteNumber);
			}
		}
		// command logic
		if (multiMessage == true) {
			multiLog();
		}
		else {
			makeSingleLog();
		}
		console.log(`Command getdiscordlog started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};