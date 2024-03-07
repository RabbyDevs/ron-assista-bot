const axios = require('axios');
// helper function: get the current date
exports.getDate = async function() {
	const today = new Date();
	const date = today.getFullYear() + '-' + (today.getMonth() + 1) + '-' + today.getDate();
	const time = today.getHours() + ':' + today.getMinutes() + ':' + today.getSeconds();
	const dateTime = date + ' ' + time;
	return dateTime;
};

async function err(interaction, error) {
	if (error == 'User not found') error = 'User was not found in Bloxlink\'s database, are you sure the user you inputted is registered with Bloxlink?'
	await interaction.followUp({ content: `There was an error while executing this command!\nPing Rabby if issue persists.\n\n\\- Error -\n${error}` });
	throw error;
};
// helper function: error the command
exports.err = async function(interaction, error) {
	if (error == 'User not found') error = 'User was not found in Bloxlink\'s database, are you sure the user you inputted is registered with Bloxlink?'
	await interaction.followUp({ content: `There was an error while executing this command!\nPing Rabby if issue persists.\n\n\\- Error -\n${error}` });
	throw error;
};

// get the ID of multiple roblox users from their usernames.
exports.robloxUsertoID = async function(interaction, robloxUserTable) {
	const response = await axios.post('https://users.roblox.com/v1/usernames/users', {
		'usernames': robloxUserTable,
		'excludeBannedUsers': false,
	})
	if (response.data.data == undefined) {
		err(`A error occured with converting Roblox USER to ID.`)
		return
	} else {
		for (const id in response.data.data[0]) {
			if (id == 'id') return response.data.data[0][id]
		}
	}
};

const { bloxlinkAPIKey } = require('../../config.json');

// Gets the RobloxID of a discord user VIA bloxlink global api.
exports.bloxlinkID = async function(interaction, userId) {
	if (userId == undefined) err(interaction, 'One User ID was invalid.')
	const response = await axios.get(`https://api.blox.link/v4/public/discord-to-roblox/${userId}`, {
		headers: {
			'Authorization': bloxlinkAPIKey
		}
	})
	if (response.data.error == undefined) {
		return response.data.robloxID;
	}
	else {
		err(interaction, response.data.error);
	}
};

// Gets the username from a Roblox ID via Roblox official API endpoints.
exports.robloxIDtoUser = async function(interaction, robloxID) {
	if (robloxID == undefined) err(interaction, 'One Roblox ID was invalid.')
	const response = await axios.get(`https://users.roblox.com/v1/users/${robloxID}`)
	if (response.data.errors == undefined) {
		return response.data.name;
	}
	else {
		err(interaction, response.data.errors.message);
	}
	return username;
};

// Gets the username from a Roblox ID via Roblox official API endpoints.
exports.robloxInfoFromID = async function(interaction, robloxID) {
	if (robloxID == undefined) err(interaction, 'One Roblox ID was invalid.')
	const response = await axios.get(`https://users.roblox.com/v1/users/${robloxID}`)
	if (response.data.errors == undefined) {
		return response.data;
	}
	else {
		err(interaction, response.data.errors.message);
	}
};

exports.robloxFriendCountFromID = async function(interaction, robloxID) {
	if (robloxID == undefined) err(interaction, 'One Roblox ID was invalid.')
	const response = await axios.get(`https://friends.roblox.com/v1/users/${robloxID}/friends`)
	if (response.data.errors == undefined) {
		return Object.keys(response.data.data).length;
	}
	else {
		err(interaction, response.data.errors.message);
	}
};

exports.robloxGroupCountFromID = async function(interaction, robloxID) {
	if (robloxID == undefined) err(interaction, 'One Roblox ID was invalid.')
	const response = await axios.get(`https://groups.roblox.com/v2/users/${robloxID}/groups/roles?includeLocked=true`)
	if (response.data.errors == undefined) {
		return Object.keys(response.data.data).length;
	}
	else {
		err(interaction, response.data.errors.message);
	}
};

exports.badgeInfoFromID = async function(interaction, robloxID, iterations) {
	if (robloxID == undefined) err(interaction, 'One Roblox ID was invalid.')
	let badgeCount = 0
	let totalWinRate = 0
	let winRate = 0
	let welcomeBadgeCount = 0
	let nextCursor = null
	const regex = /Welcome|Join|visit|play/gi
	let awarders = {}
	let getIternations = 0
	do {
		const response = await axios.get((nextCursor !== null ? `https://badges.roblox.com/v1/users/${robloxID}/badges?limit=100&sortOrder=Asc&cursor=${nextCursor}` : `https://badges.roblox.com/v1/users/${robloxID}/badges?limit=100&sortOrder=Asc`))
		if (response.data.errors == undefined) {
			badgeCount += Object.keys(response.data.data).length
			nextCursor = response.data.nextPageCursor
			for (const badgeData of response.data.data) {
				totalWinRate += badgeData.statistics.winRatePercentage
				if (regex.test(badgeData.name) == true) {
					welcomeBadgeCount += 1		
				}
				(awarders[`${badgeData.awarder.id}`] ? awarders[`${badgeData.awarder.id}`] += 1 : awarders[`${badgeData.awarder.id}`] = 1)
			}
			winRate = (totalWinRate*100)/badgeCount
			getIternations += 1
		}
		else {
			err(interaction, response.data.errors.message);
		}
		if (getIternations >= iterations || getIternations >= 10) nextCursor = null
	}
	while (nextCursor !== null)

	// Create items array
	const sortedAwarders = Object.keys(awarders).map(function(key) {
		return [key, awarders[key]];
	});
	  
	  // Sort the array based on the second element
	  sortedAwarders.sort(function(first, second) {
		return second[1] - first[1];
	});
	return [badgeCount, `${Math.round(winRate)}%`, welcomeBadgeCount, sortedAwarders]

}

exports.delay = (delayInms) => {
	return new Promise(resolve => setTimeout(resolve, delayInms));
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