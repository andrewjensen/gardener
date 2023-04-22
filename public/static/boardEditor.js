// https://blockly-demo.appspot.com/static/demos/blockfactory/index.html

const BLOCKLY_CONTAINER_ID = 'board-editor-workspace';

const LISTEN_EVENT_TYPES = [
  'move',
  'change',
];

Blockly.Blocks['board_definition'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Board Definition");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_cool_design"), "NAME");
    this.appendDummyInput()
        .appendField("audio channels")
        .appendField(new Blockly.FieldNumber(2, 0, 2, 1), "AUDIO_CHANNELS");
    this.appendStatementInput("COMPONENTS")
        .setCheck("Component")
        .appendField("components");
    this.appendStatementInput("ALIASES")
        .setCheck("Alias")
        .appendField("aliases");
    this.setColour(230);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

Blockly.Blocks['board_component_switch'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Component: Switch");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_component_name"), "NAME");
    this.appendDummyInput()
        .appendField("pin")
        .appendField(new Blockly.FieldNumber(0, 0, 100, 1), "PIN");
    this.setPreviousStatement(true, "Component");
    this.setNextStatement(true, "Component");
    this.setColour(0);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

Blockly.Blocks['board_component_analog_control'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Component: Analog Control");
    this.appendDummyInput()
        .appendField("name")
        .appendField(new Blockly.FieldTextInput("my_component_name"), "NAME");
    this.appendDummyInput()
        .appendField("pin")
        .appendField(new Blockly.FieldNumber(0, 0, 100, 1), "PIN");
    this.setPreviousStatement(true, "Component");
    this.setNextStatement(true, "Component");
    this.setColour(0);
    this.setTooltip("");
    this.setHelpUrl("");
  }
};

Blockly.Blocks['board_alias'] = {
  init: function() {
    this.appendDummyInput()
        .appendField("Alias")
    this.appendDummyInput()
        .appendField(new Blockly.FieldTextInput("my_alias"), "ALIAS")
        .appendField("is an alias of")
        .appendField(new Blockly.FieldTextInput("my_component_name"), "NAME");
    this.setPreviousStatement(true, "Alias");
    this.setNextStatement(true, "Alias");
    this.setColour(135);
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
    },
    {
      "kind": "block",
      "type": "board_alias"
    },
  ]
}

const workspace = Blockly.inject(BLOCKLY_CONTAINER_ID, {
  toolbox: toolbox,
  renderer: 'zelos',
  // renderer: 'thrasos',
});

workspace.addChangeListener(onWorkspaceUpdate);

function onWorkspaceUpdate(event) {
  if (!LISTEN_EVENT_TYPES.includes(event.type)) {
    return;
  }

  console.log('workspace updated:', event.type);

  const workspaceDom = Blockly.Xml.workspaceToDom(workspace);

  // const xmlText = Blockly.Xml.domToPrettyText(workspaceDom);
  // console.log('dom as text:', xmlText);

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

  const componentBlocks = boardDefinitionBlock.querySelectorAll('block[type^="board_component_');
  for (let componentBlock of componentBlocks) {
    const { name, definition } = parseComponent(componentBlock);
    board.components[name] = definition;
  }

  const aliasBlocks = boardDefinitionBlock.querySelectorAll('block[type="board_alias"]');
  for (let aliasBlock of aliasBlocks) {
    const { alias, name } = parseAlias(aliasBlock);
    board.aliases[alias] = name;
  }

  return board;
}

function parseComponent(blockNode) {
  const name = getBlockFieldValue(blockNode, 'NAME');
  const componentType = blockNode.getAttribute('type');

  switch (componentType) {
    case 'board_component_switch':
      return {
        name,
        definition: {
          component: 'Switch',
          pin: parseInt(getBlockFieldValue(blockNode, 'PIN'))
        }
      };
    case 'board_component_analog_control':
      return {
        name,
        definition: {
          component: 'AnalogControl',
          pin: parseInt(getBlockFieldValue(blockNode, 'PIN'))
        }
      };
    default:
      throw new Error(`Unhandled component type: ${componentType}`);
  }
}

function parseAlias(blockNode) {
  const alias = getBlockFieldValue(blockNode, 'ALIAS');
  const name = getBlockFieldValue(blockNode, 'NAME');

  return {
    alias,
    name
  };
}

function getBlockFieldValue(blockNode, fieldName) {
  const fieldNode = blockNode.querySelector(`field[name="${fieldName}"]`);
  if (!fieldNode) {
    throw new Error(`Could not get field ${fieldName}`);
  }

  return fieldNode.textContent;
}
