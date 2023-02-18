const MAX_ATTEMPTS = 60;
const ATTEMPT_INTERVAL_MS = 1000;
const FINISHED_STATUSES = ['Compiled', 'Failed'];

const POLLING_ENABLED = true;

main();

async function main() {
  const patchId = window.PATCH_ID;

  console.log('Patch ID:', patchId);

  console.log('Time to poll patch state!', patchId);

  const maxAttempts = POLLING_ENABLED ? MAX_ATTEMPTS : 1;

  let completed = false;
  for (let attempts = 0; attempts < maxAttempts; attempts++) {
    const patch = await fetchPatchMeta(patchId);
    console.log('current patch status:', patch.status);
    const statusName = getStatusName(patch.status);
    document.getElementById('status').innerHTML = getStatusMessage(statusName);

    if (FINISHED_STATUSES.includes(statusName)) {
      if (statusName === 'Compiled') {
        document.getElementById('download').classList.remove('download-disabled');
      }

      completed = true;

      break;
    }

    await pause(ATTEMPT_INTERVAL_MS);
  }

  if (!completed) {
    alert(`Patch never finished compiling after ${maxAttempts} polling attempts!`);
  }
}

async function fetchPatchMeta(patchId) {
  const response = await fetch(`/api/patches/${patchId}`);
  const responseBody = await response.json();

  return responseBody;
}

async function pause(ms) {
  return new Promise(resolve => {
    setTimeout(resolve, ms);
  });
}

function getStatusName(status) {
  let statusName = status;
  if (typeof status === 'object' && status['Failed']) {
    statusName = 'Failed';
  }

  return statusName;
}

function getStatusMessage(statusName) {
  const messages = {
    'Uploaded': 'waiting to compile...',
    'Compiling': 'compiling...',
    'Compiled': 'compiled successfully',
    'Failed': 'failed to compile!',
  };

  return messages[statusName];
}
