// defined-in-js.js

const canvas = document.getElementById('neuralNetworkCanvas');
canvas.style.width = "22rem"
canvas.style.height = "40rem"
canvas.width = "500"
canvas.height = "900"
const ctx = canvas.getContext('2d');

const layerSizes = [2, 6, 4, 1];
const spaceBetweenLayers = 220;
const neuronRadius = 30;

let init = false;
let network = [];

let neuron = (_layer, _index,  _before)=>{
    let value = 0;
    if (_before === undefined) _before = [];
    function get_line_width(gene) {
        return (gene + 16.0 ) / 10;
    }
    function get_node_color(){
        let v = (value ) / 8 * 255;
        let r = 0, g = 0;
        if ( v > 0 ){
            g = v;
        }
        else {
            r = -v;
        }
        return `rgba(${r},${g},0,1)`;
    }

    function calculate_value(genes){
        let v = 0;
        for (let i = 0; i < _before.length; i++) {
            v += _before[i].value() * genes[i];
        }
        v += genes[genes.length -1];
        value = v;
        if ( isNaN(v)){
            console.log();
        }
    }
    function _render(genes){
        if (genes === undefined){
            genes = [0,0,0];
        }
        else {
            calculate_value(genes);
        }

        let gene_counter = 0;
        let numNeurons = layerSizes[_layer];
        let verticalSpacing = (canvas.width - (numNeurons * 2 * neuronRadius)) / (numNeurons + 1);
        const y = spaceBetweenLayers * _layer + spaceBetweenLayers / 2;
        const x = verticalSpacing * (_index + 1) + (2 * _index + 1) * neuronRadius;
        let beforeNeuronLayer = _before.length;
        let nextLayerVerticalSpacing = (canvas.width - (beforeNeuronLayer * 2 * neuronRadius)) / (beforeNeuronLayer + 1);

        for (let j = 0; j < beforeNeuronLayer; j++) {
            const beforeNeuronY = (_layer - 1) * spaceBetweenLayers + spaceBetweenLayers / 2;
            const beforeNeuronX = nextLayerVerticalSpacing * (j + 1) + (2 * j + 1) * neuronRadius;

            let lineWidth = get_line_width(genes[gene_counter++]);
            ctx.beginPath();
            ctx.moveTo(x , y-neuronRadius);
            ctx.lineTo(beforeNeuronX , beforeNeuronY+neuronRadius);
            ctx.strokeStyle = 'black';
            ctx.lineWidth = lineWidth;
            ctx.stroke();
        }
        let lineWidth = get_line_width(genes[gene_counter++]);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(x, y, neuronRadius, 0, 2 * Math.PI);
        ctx.fillStyle = get_node_color();
        ctx.strokeStyle = 'rgba(0,0,0,1)';
        ctx.fill();
        ctx.lineWidth = lineWidth;
        ctx.stroke();
    }

    return {
        setValue : (v)=>{value = v},
        value : ()=>value,
        getGeneSize : ()=> {
            return _before.length + 1;
        },
        render : _render
    }
}
function initialize(){
    if ( init )return;
    init = true;

    network.push(neuron(0,0,[]));
    network.push(neuron(0,1,[]));
    network.push(neuron(1,0,[network[0],network[1]]));
    network.push(neuron(1,1,[network[0],network[1]]));
    network.push(neuron(1,2,[network[0],network[1]]));
    network.push(neuron(1,3,[network[0],network[1]]));
    network.push(neuron(1,4,[network[0],network[1]]));
    network.push(neuron(1,5,[network[0],network[1]]));
    network.push(neuron(2,0,[network[2],network[3],network[4],network[5],network[6],network[7]]));
    network.push(neuron(2,1,[network[2],network[3],network[4],network[5],network[6],network[7]]));
    network.push(neuron(2,2,[network[2],network[3],network[4],network[5],network[6],network[7]]));
    network.push(neuron(2,3,[network[2],network[3],network[4],network[5],network[6],network[7]]));
    network.push(neuron(3,0,[network[8],network[9],network[10],network[11]]));
}

initialize();
export function render( gene, posStr ) {
    let array = JSON.parse(gene);
    let pos = JSON.parse(posStr);
    network[0].setValue(pos[0]);
    network[1].setValue(pos[1]);

    // Fill the canvas with the background color
    ctx.fillStyle = 'lightblue';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    network[0].render();
    network[1].render();
    for (let i = 2; i < network.length; i++) {
        let neuron = network[i];
        let single_gene = array.splice(0,neuron.getGeneSize());
        neuron.render(single_gene);
    }
}


