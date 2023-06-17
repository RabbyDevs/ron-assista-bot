/* eslint-disable no-unused-vars */
const { SlashCommandBuilder } = require('discord.js');
const http = require('https');

async function err(interaction, error) {
	await interaction.editReply({ content: `There was an error while executing this command!\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
	throw error;
}

async function getDate() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
}

async function getRobloxIdFromUser(robloxUserTable) {
	const postData = JSON.stringify({
		'usernames': robloxUserTable,
		'excludeBannedUsers': false,
	});
	const options = {
		hostname: 'users.roblox.com',
		method: 'POST',
		path: '/v1/usernames/users',
		protocol: 'https:',
		headers: {
			'accept': 'text/json',
			'Content-Type': 'application/json',
		},
	};
	const id = new Promise((resolve, reject) => {
		const req = http.request(options, res => {
			console.log('Roblox POST request started by getRobloxIdFromUser function!');
			console.log(`STATUS: ${res.statusCode}`);
			console.log(`HEADERS: ${JSON.stringify(res.headers)}`);
			res.setEncoding('utf8');
			const data = [];
			res.on('data', (chunk) => {
				data.push(chunk);
			});
			res.on('end', () => {
				let resBody = JSON.parse(data);
				switch (res.headers['content-type']) {
				case 'text/json':
					resBody = JSON.parse(resBody);
					break;
				}
				resolve(resBody);
			});
		});
		req.on('error', reject);
		req.write(postData);
		req.end();
	});

	return id;
}

module.exports = {
	data: new SlashCommandBuilder()
		.setName('getrobloxlog')
		.setDescription('Replies with a proper RON Log when given Discord User.')
		.addStringOption(option =>
			option
				.setName('users')
				.setDescription('User(s) to make log from, use a space to separate users.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('type')
				.setDescription('Type of infraction')
				.setRequired(true)
				.setChoices(
					{ name: 'Game: Ban', value: 'Game Ban' },
					{ name: 'Game: Tempban', value: 'Temporary Game Ban' },
					{ name: 'Game: Serverban', value: 'Game Server Ban' },
					{ name: 'Game: Kick', value: 'Kick' },
					{ name: 'Game: Warn', value: 'Warn' },
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
		await interaction.editReply('Making logs, please stand-by.');
		console.log(`Command getrobloxlog begun on ${await getDate()} by ${interaction.user.username}.`);
		const users = interaction.options.getString('users').split(' ');
		const type = interaction.options.getString('type');
		const reason = interaction.options.getString('reason').split('|');
		const note = (interaction.options.getString('note') ? interaction.options.getString('note').split('|') : { undefined });
		const multiMessage = (interaction.options.getBoolean('multimessage') ? interaction.options.getBoolean('multimessage') : false);
		const duration = (type == 'Temporary Ban' ? interaction.options.getString('duration') : undefined);
		const robloxUsers = await getRobloxIdFromUser(users).catch(error => err(interaction, error));
		async function makeSingleLog() {
			let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
			for (const userData of robloxUsers.data) {
				text += `[${userData.name}:${userData.id}]\n`;
			}
			text += (note[0] ? `[${reason[0]}]\nNote: ${note[0]}` : `[${reason[0]}]`);
			await interaction.editReply(text);
		}
		async function multiLog() {
			let reasonNumber = 0;
			let noteNumber = 0;
			for (const userData of robloxUsers.data) {
				let text = (duration ? `[${type}: ${duration}]\n` : `[${type}]\n`);
				text += `[${userData.name}:${userData.id}]\n[${reason[reasonNumber]}]`;
				text += (note[noteNumber] ? `\nNote: ${note[noteNumber]}` : '');
				await interaction.followUp(text);
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				note[noteNumber] = null;
				noteNumber = (note[noteNumber + 1] ? noteNumber + 1 : noteNumber);
			}
		}
		if (multiMessage == true) {
			await interaction.editReply('Creating multiple logs, please standby!');
			multiLog();
		}
		else {
			makeSingleLog();
		}
		console.log(`Command getrobloxlog started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};