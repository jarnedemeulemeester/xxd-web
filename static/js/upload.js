//DOM
const $ = document.querySelector.bind(document);

//APP
let App = {};
App.init = (function() {
	//Init
	function handleFileSelect(evt) {
		const files = evt.target.files; // FileList object
		let output = [];

		for (var i = 0, f; f = files[i]; i++) {
			output.push('<li><strong>', escape(f.name), '</strong></li>');
		}
		$('.file-name').innerHTML = '<ul>' + output.join('') + '</ul>';
	}

	// input change
	$("input[type=file]").addEventListener("change", handleFileSelect);
})();
