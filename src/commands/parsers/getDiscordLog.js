/* eslint-disable no-unused-vars */
const { SlashCommandBuilder } = require('discord.js');
const http = require('https');

const { bloxlinkAPIKey } = require('/home/rabby/ron-assista-bot/config.json');
const { error } = require('console');

async function getDate() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
}

async function getRobloxId(userId) {
	const id = new Promise((resolve, reject) => {
		http.get(`https://api.blox.link/v4/public/discord-to-roblox/${userId}`, {
			headers: { 'Authorization': bloxlinkAPIKey },
		}, (response) => {
			const data = [];
			const headerDate = response.headers && response.headers.date ? response.headers.date : 'no response date';
			console.log('Bloxlink GET request started by getRobloxId!');
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
					error(obj.error);
					reject(obj.error);
				}
			});
		});
	});
	return id;
}

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
				.setRequired(false),
		),
	async execute(interaction) {
		console.log(`Command getdiscordlog begun on ${await getDate()} by ${interaction.user.username}, with parameters: ${interaction.options.getString('ids')}, ${interaction.options.getString('type')}, ${interaction.options.getString('reason')}, ${interaction.options.getString('note')}, ${interaction.options.getBoolean('multimessage')}.`);
		const users = interaction.options.getString('ids').split(' ');
		const type = interaction.options.getString('type');
		const reason = interaction.options.getString('reason').split('|');
		const note = (interaction.options.getString('note') ? interaction.options.getString('note').split('|') : { undefined });
		const multiMessage = (interaction.options.getBoolean('multimessage') ? interaction.options.getBoolean('multimessage') : false);
		async function makeSingleLog() {
			let text = '';
			text = `[${type}]\n`;
			for (const id of users) {
				if (type == 'Ban') {
					const robloxId = await getRobloxId(id);
					const robloxUser = await getUserFromRobloxId(await robloxId);
					text += `[<\\@${id}>:${id}:${robloxUser}:${robloxId}]\n`;
				}
				else {
					text += `[<\\@${id}>:${id}]\n`;
				}
			}
			text += (note[0] ? `[${reason[0]}]\nNote: ${note[0]}` : `[${reason[0]}]`);
			await interaction.editReply(text);
		}
		async function multiLog() {
			let reasonNumber = 0;
			let noteNumber = 0;
			for (const id of users) {
				if (type == 'Ban') {
					const robloxId = await getRobloxId(id);
					const robloxUser = await getUserFromRobloxId(await robloxId);
					const textNoNote = `[${type}]\n[<\\@${id}>:${id}:[${robloxUser}]:${robloxId}]\n[${reason[reasonNumber]}]`;
					const textWithNote = `[${type}]\n[<\\@${id}>:${id}:[${robloxUser}]:${robloxId}]\n[${reason[reasonNumber]}]\nNote: ${note[noteNumber]}`;
					const followUp = (note[noteNumber] ? textWithNote : textNoNote);
					await interaction.followUp(followUp);
				}
				else {
					const textNoNote = `[${type}]\n[<\\@${id}>:${id}]\n[${reason[reasonNumber]}]`;
					const textWithNote = `[${type}]\n[<\\@${id}>:${id}]\n[${reason[reasonNumber]}]\nNote: ${note[noteNumber]}`;
					const followUp = (note[noteNumber] ? textWithNote : textNoNote);
					await interaction.followUp(followUp);
				}
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				note[noteNumber] = null;
				noteNumber = (note[noteNumber + 1] ? noteNumber + 1 : noteNumber);
			}
		}
		async function commandLogic() {
			if (multiMessage == true) {
				await interaction.editReply('Creating multiple logs, please standby!');
				multiLog();
			}
			else {
				makeSingleLog();
			}
		}
		await interaction.deferReply();
		switch (type) {
		case 'Warn': {
			commandLogic();
			break;
		}
		case 'Ban': {
			commandLogic();
			break;
		}
		}
		console.log(`Command getdiscordlog started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};