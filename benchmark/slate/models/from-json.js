/* eslint-disable react/jsx-key */

const { Value } = require('slate')

module.exports.default = function(json) {
  Value.fromJSON(json)
}

const input = {
  document: {
    nodes: Array.from(Array(10)).map(() => ({
      object: 'block',
      type: 'quote',
      nodes: [
        {
          object: 'block',
          type: 'paragraph',
          nodes: [
            {
              object: 'text',
              text: 'This is editable ',
            },
            {
              object: 'text',
              text: 'rich',
              marks: [{ type: 'bold' }],
            },
            {
              object: 'text',
              text: ' text, ',
            },
            {
              object: 'text',
              text: 'much',
              marks: [{ type: 'italic' }],
            },
            {
              object: 'text',
              text: ' better than a textarea!',
            },
          ],
        },
      ],
    })),
  },
}

module.exports.input = function() {
  return input
}