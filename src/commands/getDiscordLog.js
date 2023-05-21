/* eslint-disable no-unused-vars */
const { SlashCommandBuilder } = require('discord.js');
const http = require('https');

const { bloxlinkAPIKey } = require('/home/rabby/ron-assista-bot/config.json');

async function getDate() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
}

async function getRobloxId(userId) {
	const id = new Promise((resolve, reject) => {
		http.get(`https://v3.blox.link/developer/discord/${userId}?guildId=GuildIdHere`, {
			headers: { 'api-key': bloxlinkAPIKey },
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
				if (obj.success == true) {
					resolve(obj.user.robloxId);
				}
				else {
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
				.setName('reason')
				.setDescription('Reason for log if multimessage is set to true you can use "|" to separate logs for each log message.')
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
		.addBooleanOption(option =>
			option
				.setName('multimessage')
				.setDescription('Should the bot split logs into multiple messages if there are multiple users?')
				.setRequired(true),
		),
	async execute(interaction) {
		console.log(`Command getdiscordlog begun on ${await getDate()} by ${interaction.user.username}.`);
		const users = interaction.options.getString('ids').split(' ');
		const type = interaction.options.getString('type');
		const reason = interaction.options.getString('reason').split('|');
		const multiMessage = interaction.options.getBoolean('multimessage');
		async function makeSingleLog() {
			let text = '';
			text = `Log from <@${interaction.user.id}>.\n[${type}]\n`;
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
			text += `[${reason}]`;
			return text;
		}
		async function multiLog() {
			let reasonNumber = 0;
			for (const id of users) {
				if (type == 'Ban') {
					const robloxId = await getRobloxId(id);
					const robloxUser = await getUserFromRobloxId(await robloxId);
					await interaction.followUp(`[${type}]\n[<\\@${id}>:${id}:[${robloxUser}]:${robloxId}]\n[${reason[reasonNumber]}]`);
				}
				else {
					await interaction.followUp(`[${type}]\n[<\\@${id}>:${id}]\n[${reason[reasonNumber]}]`);
				}
				if (reason[reasonNumber + 1] != undefined) {
					reasonNumber = reasonNumber + 1;
				}
			}
		}
		async function commandLogic() {
			if (multiMessage == true) {
				await interaction.editReply('Creating multiple logs, please standby!');
				multiLog();
			}
			else {
				await interaction.editReply(await makeSingleLog());
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
		console.log(`Command getdiscordlog started by ${interaction.username} ended on ${await getDate()}`);
	},
};