// https://blockly-demo.appspot.com/static/demos/blockfactory/index.html

const BLOCKLY_CONTAINER_ID = 'board-editor-workspace';

console.log('Hello from the board editor');

Blockly.Blocks['board_definition'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("board definition");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_cool_design"), "NAME");
    this.appendDummyInput()
        .appendField("audio channels")
        .appendField(new Blockly.FieldNumber(2, 0, 2, 1), "AUDIO_CHANNELS");
    this.appendStatementInput("COMPONENTS")
        .setCheck(null)
        .appendField("components");
    this.appendStatementInput("ALIASES")
        .setCheck(null)
        .appendField("aliases");
    this.setColour(230);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

Blockly.Blocks['board_component_switch'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Switch");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_component_name"), "NAME");
    this.appendDummyInput()
        .appendField("pin")
        .appendField(new Blockly.FieldNumber(0, 0, 100, 1), "PIN");
    this.setPreviousStatement(true, null);
    this.setNextStatement(true, null);
    this.setColour(0);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

Blockly.Blocks['board_component_analog_control'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Analog Control");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_component_name"), "NAME");
    this.appendDummyInput()
        .appendField("pin")
        .appendField(new Blockly.FieldNumber(0, 0, 100, 1), "PIN");
    this.setPreviousStatement(true, null);
    this.setNextStatement(true, null);
    this.setColour(0);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

const toolbox = {
  "kind": "flyoutToolbox",
  "contents": [
    {
      "kind": "block",
      "type": "board_definition"
    },
    {
      "kind": "block",
      "type": "board_component_switch"
    },
    {
      "kind": "block",
      "type": "board_component_analog_control"
    }
  ]
}

const workspace = Blockly.inject(BLOCKLY_CONTAINER_ID, {toolbox: toolbox});

workspace.addChangeListener(onWorkspaceUpdate);

function onWorkspaceUpdate(event) {
  // TODO: filter out lots of event types to avoid re-renders

  console.log('workspace updated');

  const workspaceDom = Blockly.Xml.workspaceToDom(workspace);
  console.log('dom:', workspaceDom);

  const xmlText = Blockly.Xml.domToPrettyText(workspaceDom);
  console.log('dom as text:', xmlText);

  const boardJson = domToBoardJson(workspaceDom);

  document.getElementById('board-editor-json-output').innerText = JSON.stringify(boardJson, null, 2);
}

function domToBoardJson(workspaceDom) {
  const board = {
    name: 'TODO',
    som: 'seed',
    defines: {},
    display: {},
    audio: {
      channels: 0
    },
    components: {},
    aliases: {}
  };

  const boardDefinitionBlock = workspaceDom.querySelector('block[type="board_definition"]');
  if (!boardDefinitionBlock) {
    return {};
  }

  board.name = getBlockFieldValue(boardDefinitionBlock, 'NAME')
  board.audio.channels = parseInt(getBlockFieldValue(boardDefinitionBlock, 'AUDIO_CHANNELS'))

  return board;
}

function getBlockFieldValue(blockNode, fieldName) {
  const fieldNode = blockNode.querySelector(`field[name="${fieldName}"]`);
  if (!fieldNode) {
    throw new Error(`Could not get field ${fieldName}`);
  }

  return fieldNode.textContent;
}
