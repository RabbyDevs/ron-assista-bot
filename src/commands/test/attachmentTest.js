const { SlashCommandBuilder } = require('discord.js');

module.exports = {
	data: new SlashCommandBuilder()
		.setName('attachment')
		.setDescription('10101010101')
		.addAttachmentOption(option =>
			option
				.setName('attachment')
				.setDescription('beepboopbapbappbapobao.')
				.setRequired(true)),
	async execute(interaction) {
		const attachment = await interaction.options.getAttachment('attachment');
		console.log(await attachment.name);
		await interaction.deferReply();
		await interaction.editReply('yeah no this is a test');
	},
};