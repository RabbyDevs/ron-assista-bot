/* eslint-disable no-unused-vars */
const { SlashCommandBuilder, PermissionFlagsBits } = require('discord.js');
const http = require('https');

const { bloxlinkAPIKey } = require('/home/rabby/ron-assista-bot/config.json');

// helper function: error the command
async function err(interaction, error) {
	await interaction.followUp({ content: `**There was an error while executing this command!**\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
	throw error;
}

// helper function: get the current date.
async function getDate() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = [date + ' ' + time, today.getTime()];
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
		.setName('probationlog')
		.setDescription('Replies with the probation log given a Discord ID or multiple.')
		.addStringOption(option =>
			option
				.setName('ids')
				.setDescription('ID(s) to make log from, use a space to separate users.')
				.setRequired(true))
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
				.setName('timeformat')
				.setDescription('Time format of probation (look up a guide on google, defaults to f).')
				.setRequired(false))
		.setDefaultMemberPermissions(PermissionFlagsBits.KickMembers),
	async execute(interaction) {
		await interaction.deferReply();
		// detect if the user is on mobile on any platform:
		const isMobile = (await interaction.member.presence.clientStatus.mobile ? true : false);
		(isMobile == true ? await interaction.editReply('Mobile detected! Adding mobile friendly log(s).') : await interaction.editReply('Making log(s), please stand-by!'));
		console.log(`Command getdiscordlog begun on ${await getDate()[0]} by ${interaction.user.username}, with parameters: ${interaction.options.getString('ids')}, ${interaction.options.getString('type')}, ${interaction.options.getString('reason')}, ${interaction.options.getString('note')}, ${interaction.options.getBoolean('multimessage')}.`);
		// variables/arguments
		const users = interaction.options.getString('ids').split(' ');
		const reason = interaction.options.getString('reason').split('|');
		const duration = interaction.options.getString('duration').split('|');
		const timeFormat = (interaction.options.getString('timeFormat') ? interaction.options.getString('timeFormat').split(' ') : 'f');
		// possibly split at a space, then parse by checking the ending letter against a dictionary of items?
		// okk now checking how to check the last letter of a string :3
		const calculatedDurations = [];
		let durNumber = 0;
		for (const dateID in duration) {
			// epoch part
			const dates = { 'h': 3600, 'd': 86400, 'w': 604800, 'm': 2629743 };
			const date = duration[dateID].replace(/ /gi, '');
			const numbers = date.split(/[dwmh]/gi);
			numbers.pop();
			const letters = date.split(/[1234567890]/gi);
			letters.shift();

			const today = Date.now();
			const todayEpoch = Math.floor(today / 1000);
			let epoch = Math.floor(today / 1000);
			for (const number in numbers) {
				const combinedNumber = numbers[number] * dates[letters[number]];
				epoch = epoch + combinedNumber;
			}

			// full duration part
			const fullMonths = { 'h': 'Hour', 'd': 'Day', 'w': 'Week', 'm': 'Month' };
			let fullDuration = '';
			for (const letter in letters) {
				let month = (numbers[letter] > 1 ? `${fullMonths[letters[letter]]}s` : `${fullMonths[letters[letter]]}`);
				month += (letter == letters.length - 2 || letter == letters.length - 1 ? (letter == letters.length - 2 ? ' and' : '') : ',');
				fullDuration += `${numbers[letter]} ${month} `;
			}
			calculatedDurations[durNumber + 1] = todayEpoch;
			calculatedDurations[durNumber + 2] = epoch;
			calculatedDurations[durNumber] = fullDuration;
			durNumber = durNumber + 3;
		}

		// make a multiple msg log from arguments + table magic
		async function multiLog() {
			let reasonNumber = 0;
			let durationNumber = 0;
			for (const id of users) {
				// make a log
				const robloxId = await getRobloxId(id).catch(error => err(interaction, error));
				const robloxUser = await getUserFromRobloxId(await robloxId).catch(error => err(interaction, error));
				let text = '';
				text += `[<\\@${id}> - ${id} - ${robloxUser}:${robloxId}]\n\n`;
				text += `[${reason[reasonNumber]}]\n\n`;
				text += `[${calculatedDurations[durationNumber]}(<t\\:${calculatedDurations[durationNumber + 1]}:${timeFormat}> - <t\\:${calculatedDurations[durationNumber + 2]}:${timeFormat}>)]`;
				await interaction.followUp((isMobile == true ? 'Desktop version of the log: text' + text : text));
				(isMobile == true ? await interaction.followUp(text.replace(/[\\]/gi, '')) : undefined);
				reasonNumber = (reason[reasonNumber + 1] ? reasonNumber + 1 : reasonNumber);
				durationNumber = (calculatedDurations[durationNumber + 3] ? durationNumber + 3 : durationNumber);
			}
		}
		// command logic
		multiLog();
		console.log(`Command getdiscordlog started by ${interaction.user.username} ended on ${await getDate()[0]}`);
	},
};