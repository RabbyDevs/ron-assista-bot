const { SlashCommandBuilder, PermissionFlagsBits, EmbedBuilder } = require('discord.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('postcreator')
		.setDescription('Makes a forum post, with an embed.')
        .addStringOption(option =>
			option
				.setName('forum')
				.setDescription('What ID is the forum your sending this to.')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('title')
				.setDescription('Title of embed, separate with "|".')
				.setRequired(true))
		.addStringOption(option =>
			option
				.setName('body')
				.setDescription('Description or body of embed, separate with "|" input custom to initiate custom formatting.')
				.setRequired(true))
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
        console.log(`Command ${interaction.commandName} begun on ${await getDate()} by ${interaction.user.username}.`);
		await interaction.deferReply();
		await interaction.editReply('Making and sending embeds...');
		const titles = interaction.options.getString('title').split('|');
		let bodies = interaction.options.getString('body').split('|');
        const forum = interaction.options.getString('forum');
        let done = false
    
		let bodyNumber = 0;
        let titleNumber = 0;

        async function forLoop() {
            return new Promise(async (resolve, reject) =>{
                for (const title of titles) {
                    if (bodies[bodyNumber] == 'custom') {
                        const collectorFilter = m => m.author.id.includes(interaction.user.id);
                        const collector = interaction.channel.createMessageCollector({ filter: collectorFilter, time: 15000 });
                        await interaction.followUp(`Input your custom body for "${title}".`);
        
                        collector.on('collect', m => {
                            collector.stop()
                            interaction.followUp('Input received!')
                            bodies[bodyNumber] = m.content
                            interaction.guild.channels.fetch(forum) 
                            .then(channel => {
                                // inside a command, event listener, etc.
                                const embed = new EmbedBuilder()
                                .setColor(0x6E1C09)
                                .setTitle(title)
                                .setDescription(bodies[bodyNumber])
                                .setTimestamp()
                                        
                                channel.threads.create({
                                    name: title,
                                    autoArchiveDuration: 4320,
                                        message: {
                                            embeds: [embed],
                                        },
                                        reason: title,
                                    })
                            });
                            if (titleNumber == titles.length) {
                                resolve('Done!')
                            }
                        });
                    }
                    else {
                        interaction.guild.channels.fetch(forum) 
                        .then(channel => {
                            // inside a command, event listener, etc.
                            const embed = new EmbedBuilder()
                            .setColor(0x6E1C09)
                            .setTitle(title)
                            .setDescription(bodies[bodyNumber])
                            .setTimestamp()
                                    
                            channel.threads.create({
                                name: title,
                                autoArchiveDuration: 4320,
                                    message: {
                                        embeds: [embed],
                                    },
                                    reason: title,
                                })
                        });
                        if (titleNumber == titles.length) {
                            resolve('Done!')
                        }
                    }
                    bodyNumber = (bodyNumber[bodyNumber + 1] ? bodyNumber + 1 : bodyNumber);
                    titleNumber = titleNumber + 1
                }
            })
        }
        forLoop().then(async returned => {
            await interaction.followUp(returned)
        })
        console.log(`Command ${interaction.commandName} started by ${interaction.user.username} ended on ${await getDate()}`);
	},
};