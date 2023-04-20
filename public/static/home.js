main();

async function main() {
  console.log('main()');
  document.querySelector('[name="board"]').addEventListener('change', (event) => {
    onSelectBoard(event.target.value);
  })
}

function onSelectBoard(boardValue) {
  console.log('onSelectBoard', boardValue);

  if (boardValue === 'seed') {
    document.getElementById('board-def-container').classList.remove('hidden');
  } else {
    document.getElementById('board-def-container').classList.add('hidden');
  }
}
