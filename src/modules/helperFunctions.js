// helper function: get the current date
exports.getDate = async function() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
};

// helper function: error the command
exports.err = async function(interaction, error) {
	await interaction.followUp({ content: `There was an error while executing this command!\n<@744076526831534091> Error:\n${error}`, ephemeral: true });
	throw error;
};

const http = require('https');

// get the ID of multiple roblox users from their usernames.
exports.robloxUsertoID = async function(robloxUserTable) {
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
			console.log('Roblox POST request started by robloxUsertoID function!');
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
};

const { bloxlinkAPIKey } = require('/home/rabby/ron-assista-bot/config.json');

// Gets the RobloxID of a discord user VIA bloxlink global api.
exports.bloxlinkID = async function(userId) {
	const id = new Promise((resolve, reject) => {
		http.get(`https://api.blox.link/v4/public/discord-to-roblox/${userId}`, {
			headers: { 'Authorization': bloxlinkAPIKey },
		}, (response) => {
			const data = [];
			const headerDate = response.headers && response.headers.date ? response.headers.date : 'no response date';
			console.log(`Bloxlink GET request started by robloxID. Input is: ${userId}`);
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
};

// Gets the username from a Roblox ID via Roblox official API endpoints.
exports.robloxIDtoUser = async function(robloxId) {
	const username = new Promise((resolve, reject) => {
		http.get(`https://users.roblox.com/v1/users/${robloxId}`, (response) => {
			const data = [];
			const headerDate = response.headers && response.headers.date ? response.headers.date : 'no response date';
			console.log('Roblox GET request started by robloxUserfromID!');
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
};

// duration calculation
exports.calculateDuration = async function(duration) {
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
		calculatedDurations[durNumber] = (date.charAt(0) == '0' ? undefined : fullDuration);
		calculatedDurations[durNumber + 1] = (date.charAt(0) == '0' ? undefined : todayEpoch);
		calculatedDurations[durNumber + 2] = (date.charAt(0) == '0' ? undefined : epoch);
		durNumber = durNumber + 3;
	}
	return calculatedDurations;
};