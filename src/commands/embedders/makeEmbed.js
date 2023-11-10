const { SlashCommandBuilder, PermissionFlagsBits, EmbedBuilder, Client, ThreadAutoArchiveDuration } = require('discord.js');

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
				.setDescription('Description or body of embed, separate with "|"')
				.setRequired(true))
		.setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers),
	async execute(interaction) {
		await interaction.deferReply();
		await interaction.editReply('Making and sending embeds...');
		const titles = interaction.options.getString('title').split('|');
		const bodies = interaction.options.getString('body').split('|');
        const forum = interaction.options.getString('forum');
    
		let bodyNumber = 0;
		for (const title of titles) {
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
			bodyNumber = (bodyNumber[bodyNumber + 1] ? bodyNumber + 1 : bodyNumber);
		}
		await interaction.followUp('Done!');
	},
};