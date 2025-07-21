var gridSize = 10;
var MAX_MEM = 5;
let agList = {};
var globalidx = gridSize*gridSize;

var breakFlag = false;

//(q-learning h-params)
var alpha = 0.2;
var gamma = 0.95;
var epsilon = 0.1;

var totRounds = 1e05;
var n = 0;
var colorMap = {};
var maxLimit = 0.0;
var saveBuffer = {};
//canvas setup
const canvasSize = 600; // Fixed canvas size in pixels
var outSize = canvasSize; // Always match canvas size
var uniqueAgents = {};
var u_ftn_map = {};
var m_ftn_map = {};

var u_coop = {};
var m_coop = {};

var stress_map = {};
var uniCellularFitness = [];
var multiCellularFitness = [];

var uniCooperation = [];
var multiCooperation = [];

var stress = 0.0;

//default dict (req. when you reset)
var defaults = {'n_agents' : 10,
                'memory': 5,
                'alpha': 0.2,
                'gamma': 0.95,
                'epsilon': 0.1,};


//playback control bar (creation)
var playicon = document.createElement("button");
playicon.textContent = "\u25B6";
// playicon.style = "font-size:1.5em";
playicon.id = "playicon";

var pauseicon = document.createElement("button");
pauseicon.textContent = "\u23F8";
// pauseicon.style = "font-size:2.0em";
pauseicon.id = "pauseicon";

var reseticon = document.createElement("button");
reseticon.textContent = "\u21BA";
// reseticon.style = "font-size:2.0em";
reseticon.id = "reseticon";

var slider = document.createElement("input");
slider.type="range";
slider.id="slider";
slider.min="0";
slider.max="100";
slider.step="1";
slider.value="0";

//main div
var main_div = document.createElement('div');
main_div.className = "main_simulator_div";
//fetch story div
var story_div = document.getElementsByClassName('story_div');
story_div[0].appendChild(main_div);

//add to display
var playbackctrl = document.createElement("div");
playbackctrl.id = "playbackcontrols";
main_div.appendChild(playbackctrl)

playbackctrl.appendChild(playicon);
playbackctrl.appendChild(pauseicon);
playbackctrl.appendChild(reseticon);
playbackctrl.appendChild(slider);

//information bar
var infodiv = document.createElement('div');
infodiv.className = "info_div";
var infotext = document.createElement('span');
infotext.className = "info_text";
infotext.textContent = "Click \u25B6 to Start"
infodiv.appendChild(infotext);

main_div.appendChild(infodiv);

//canvas+colorbar div
var canvas_cbar_div = document.createElement('div');
canvas_cbar_div.className = "canvas_cbar_div";
main_div.appendChild(canvas_cbar_div);

//create canvas
//canvas div
var canvas_div = document.createElement('div');
canvas_div.className = "canvas_div";
canvas_cbar_div.appendChild(canvas_div);

var canvas = document.createElement('canvas');
canvas.id = "canvas";
canvas.width = canvasSize;
canvas.height = canvasSize;
canvas_div.appendChild(canvas);
var ctx = canvas.getContext('2d');
ctx.imageSmoothingEnabled = false;
ctx.font = "15px Garamond";
var imgData = ctx.createImageData(canvasSize, canvasSize);

//grid color
const grid_color = `rgb(10, 10, 10)`;

//colorbar div
var cbar_div = document.createElement('div');
cbar_div.className = "cbar_div";
canvas_cbar_div.appendChild(cbar_div);


//colorbar canvas
var canvas_colorbar = document.createElement('canvas');
canvas_colorbar.id = "canvas_colorbar";
canvas_colorbar.width = canvas.style.width + 300;
canvas_colorbar.height = canvas.style.height;
cbar_div.appendChild(canvas_colorbar);

var ctx_colorbar = canvas_colorbar.getContext('2d');

//graph canvas
var graph_div = document.createElement('div');
graph_div.className = "graph_div";
main_div.appendChild(graph_div);

///
var ag_graph_div = document.createElement('div');
ag_graph_div.className = "quadrant g-agent-size";
graph_div.appendChild(ag_graph_div);

var ag_graph_canvas = document.createElement('canvas');
ag_graph_canvas.id = "ag-chart";
ag_graph_div.appendChild(ag_graph_canvas);

///
var ftn_graph_div = document.createElement('div');
ftn_graph_div.className = "quadrant g-ftn";
graph_div.appendChild(ftn_graph_div);

var ag_ftn_canvas = document.createElement('canvas');
ag_ftn_canvas.id = "ftn-chart";
ftn_graph_div.appendChild(ag_ftn_canvas);

///
var coop_graph_div = document.createElement('div');
coop_graph_div.className = "quadrant g-coop";
graph_div.appendChild(coop_graph_div);

var ag_coop_canvas = document.createElement('canvas');
ag_coop_canvas.id = "coop-chart";
coop_graph_div.appendChild(ag_coop_canvas);

//stress div
var stress_graph_div = document.createElement('div');stress_graph_div.className = "quadrant g-stress";
graph_div.appendChild(stress_graph_div);

var ag_stress_canvas = document.createElement('canvas');
ag_stress_canvas.id = "stress-chart";
stress_graph_div.appendChild(ag_stress_canvas);


///chart options
const ctx1 = document.getElementById('ag-chart').getContext('2d');
const ctx2 = document.getElementById('ftn-chart').getContext('2d');
const ctx3 = document.getElementById('coop-chart').getContext('2d');
const ctx4 = document.getElementById('stress-chart').getContext('2d');

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  };

let ts_labels = [];
let sz_values = [];
const ag_plot_chart = new Chart(ctx1, {
  type: 'line',
  data: {
    labels: ts_labels,
    datasets: [{
      label: 'Avg. Size',
      data: sz_values,
      backgroundColor: 'rgb(56, 163, 165, 1)',
    }]
  },
  options: {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        title: {
        display: true,
        text: 'TimeStep',
      }
      },
      y: {
        title: {
          display: true,
          text: "Avg. Agent Size"
        }
      }
    }
  }
});

const ftn_plot_chart = new Chart(ctx2, {
  type: 'line',
  data: {
    labels: ts_labels,
    datasets: [{
      label: 'Unicellular Agents',
      data: sz_values,
      backgroundColor: 'rgba(34, 46, 80, 1)',
    },
  {
    label: 'Multicellular Agents',
    data: sz_values,
    backgroundColor: 'rgba(139, 177, 116, 1)',
  }]
  },
  options: 
  { 
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        title: {
        display: true,
        text: 'TimeStep',
      }
      },
      y: {
        title: {
          display: true,
          text: "Fitness"
        }
      }
    }
  }
});

const coop_plot_chart = new Chart(ctx3, {
  type: 'line',
  data: {
    labels: ts_labels,
    datasets: [{
      label: 'Unicellular Agents',
      data: sz_values,
      backgroundColor: 'rgba(34, 46, 80, 1)',
    },
  {
    label: 'Multicellular Agents',
    data: sz_values,
    backgroundColor: 'rgba(139, 177, 116, 1)',
  }]
  },
  options: {
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        title: {
        display: true,
        text: 'TimeStep',
      }
      },
      y: {
        title: {
          display: true,
          text: "%. Cooperating"
        }
      }
    }
  },
});

const stress_plot_chart = new Chart(ctx4, {
  type: 'line',
  data: {
    labels: ts_labels,
    datasets: [{
      label: 'Stress due to Defection', 
      data: sz_values,
      backgroundColor: 'red',
    },
  ]
  },
  options: { 
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        title: {
        display: true,
        text: 'TimeStep',
      }
      },
      y: {
        title: {
          display: true,
          text: "Stress"
        }
      }
    }
  },
});

pauseicon.hidden = true; //init 
reseticon.hidden = true;

//render grid lines
function drawGrid(lineWidth, cellWidth, cellHeight) {
  ctx.strokeStyle = grid_color;
  ctx.lineWidth = lineWidth;
  const width = canvas.width;
  const height = canvas.height;

  // Draw vertical lines
  for (let x = 0; x <= width; x += cellWidth) {
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
    ctx.stroke();
  }

  // Draw horizontal lines
  for (let y = 0; y <= height; y += cellHeight) {
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }
}

//colorbar settings
const cbar_border = 'black';
var cbar_box_width = 20;
var cbar_box_height = 10;
var box_spacing = 5;
var width_multipler = 45;
var left_offset = 5;
var top_offset = 13;

//avg fn
const mean_fn = array => array.reduce((a, b) => a + b) / array.length;
var sz_map = {};

function update_graph_values(framenumber=-1){
  //track x axis,
  //fetch agentssize
  const sz_list = Object.values(uniqueAgents);  
  var kys = [];
  var vls = [];

  if (sz_list.length){
    //if sim results exist, consult the szmap
    if (framenumber != -1){
      //rewind to a specific frame
      kys = Object.keys(sz_map);
      vls = Object.values(sz_map);
      ag_plot_chart.data.labels = kys.slice(0, framenumber+2);
      ag_plot_chart.data.datasets[0].data = vls.slice(0, framenumber+2);
      
      //update fitness data
      ftn_plot_chart.data.labels = kys.slice(0, framenumber+2);
      ftn_plot_chart.data.datasets[0].data = Object.values(u_ftn_map).slice(0, framenumber+2); 
      ftn_plot_chart.data.datasets[1].data = Object.values(m_ftn_map).slice(0, framenumber+2); 

      //update cooperability
      coop_plot_chart.data.labels = kys.slice(0, framenumber+2);
      coop_plot_chart.data.datasets[0].data = Object.values(u_coop).slice(0, framenumber+2); 
      coop_plot_chart.data.datasets[1].data = Object.values(m_coop).slice(0, framenumber+2); 

      //update stress
      stress_plot_chart.data.labels = kys.slice(0, framenumber+2);
      stress_plot_chart.data.datasets[0].data = Object.values(stress_map).slice(0, framenumber+2); 

      ag_plot_chart.update('none');
      ftn_plot_chart.update('none');
      coop_plot_chart.update('none');
      stress_plot_chart.update('none');
    }
    else{
      //normal simulation
      sz_map[n] = mean_fn(sz_list.filter(onlyUnique));
      //use updated keys and vals
      kys = Object.keys(sz_map);
      vls = Object.values(sz_map);

      //update fitness
      if (uniCellularFitness.length){
        u_ftn_map[n] = mean_fn(uniCellularFitness);
      }
      else{
        u_ftn_map[n] = 0.0;
      }
      if (multiCellularFitness.length){
        m_ftn_map[n] = mean_fn(multiCellularFitness);
      }
      else{
        m_ftn_map[n] = 0.0;
      }
      
      //update coperability
      if(uniCooperation.length){
        u_coop[n] = mean_fn(uniCooperation); 
      }
      else{
        u_coop[n] = 0.0;
      }
      if (multiCooperation.length){
        m_coop[n] = mean_fn(multiCooperation); 
      }
      else{
        m_coop[n] = 0.0;
      }

      //update stress
      stress_map[n] = stress;

      //update size data
      ag_plot_chart.data.labels = kys;
      ag_plot_chart.data.datasets[0].data = vls; 
      // console.log(vls);

      //update fitness data
      ftn_plot_chart.data.labels = kys;
      ftn_plot_chart.data.datasets[0].data = Object.values(u_ftn_map); 
      ftn_plot_chart.data.datasets[1].data = Object.values(m_ftn_map); 

      //update cooperability
      coop_plot_chart.data.labels = kys;
      coop_plot_chart.data.datasets[0].data = Object.values(u_coop); 
      coop_plot_chart.data.datasets[1].data = Object.values(m_coop); 

      //update stress
      stress_plot_chart.data.labels = kys;
      stress_plot_chart.data.datasets[0].data = Object.values(stress_map); 

      ag_plot_chart.update('none');
      ftn_plot_chart.update('none');
      coop_plot_chart.update('none');
      stress_plot_chart.update('none');
    } 
  }
}

function colorbar(){
//get available agents
//render their colors
  const uqlist = Object.keys(uniqueAgents);  
  // console.log(uqlist);

  ctx_colorbar.clearRect(0, 0, canvas_colorbar.width, canvas_colorbar.height)
  canvas_colorbar.height = canvas.height + 4;
  var ypos = top_offset; //px
  var xpos = left_offset;
  var textbuffer_x = 25;
  var textbuffer_y = 9;

  var cbar_title = "Agent Size";
  //colorbar text
  if(gridSize<6){
    ctx_colorbar.font = "10px Garamond";
    ctx_colorbar.fillText(cbar_title, 8, 9);
    ctx_colorbar.fillText("_________", 8, 10);
  }
  else{
    ctx_colorbar.font = "12px Garamond";
    ctx_colorbar.fillText(cbar_title, 8, 10);
    ctx_colorbar.fillText("_________", 8, 11.5);
  }

  for(let i =0; i<uqlist.length; i++){
    var ag_size = uqlist[i];
    var clr = colorMap[ag_size];

    //render into a separate canvas 
    ctx_colorbar.fillStyle = `rgb(
      ${Math.floor(255*clr[0])}
      ${Math.floor(255*clr[1])}
      ${Math.floor(255*clr[2])}
    )` 
    ctx_colorbar.fillRect(xpos, (ypos*2), cbar_box_width, cbar_box_height);

    ctx_colorbar.fillStyle = 'rgb(0, 0, 0)';
    if(gridSize <6){
      ctx_colorbar.font = "10px Garamond";
      textbuffer_x = 20;
      textbuffer_y = 5;
    }
    else{
      ctx_colorbar.font = "12px Garamond";
    }
    ctx_colorbar.fillText(String(ag_size), xpos+textbuffer_x, (ypos*2)+textbuffer_y);
 
    ctx_colorbar.strokeStyle = `rgb(200, 220, 200)`;
    ctx_colorbar.lineWidth = 1.0;
    ctx_colorbar.strokeRect(xpos, (ypos*2), cbar_box_width, cbar_box_height)

    ypos += box_spacing;
    ctx_colorbar.fillStyle = "white";
    ctx_colorbar.fillRect(xpos, (ypos*2), cbar_box_width, cbar_box_height);
    
    ypos += box_spacing;
    if (ypos > canvas_colorbar.height/2.5){
      //create a new column
      ypos = top_offset;
      xpos += width_multipler; 
    } 
  }
}

//seed code
//rng fn (from stackoverflow);
function jsf32(a, b, c, d) {
  return function() {
    a |= 0; b |= 0; c |= 0; d |= 0;
    let t = a - (b << 27 | b >>> 5) | 0;
    a = b ^ (c << 17 | c >>> 15);
    b = c + d | 0;
    c = d + t | 0;
    d = a + t | 0;
    return (d >>> 0) / 4294967296;
  }
}

const seedgen = () => (13242)>>>0;
const getRand = jsf32(seedgen(), seedgen(), seedgen(), seedgen());

//grid init
function initializeGrid(){
  globalidx = gridSize*gridSize;
  breakFlag = false;

  playicon.hidden=false;
  pauseicon.hidden=true;

  //replay buffer
  n = 0;
  colorMap = {};
  maxLimit = 0;
  saveBuffer = {};
  slider.value = 0;
  agList = {};

  //sz map, uq-list
  uniqueAgents = {};
  sz_map = {};

  u_ftn_map = {};
  m_ftn_map = {};

  u_coop = {};
  m_coop = {};

  stress_map = {}
  stress = 0.0;

  for (var ik =0; ik< gridSize*gridSize; ik++){
    agList[ik] = {
      parentlst : [],
      childlst : [],
      fitness: 0.001, 
      memlist : [],
      nbrs: getNeighbors(ik),
      policy: {
        '': getrandompolicy(), 
      },
      mem_size: Math.floor(getRand() * MAX_MEM)
    };
  } 
  //save first frame
  saveBuffer[0] = agList;
  colorMap[1] = spectralColor[1];
}

//visualization code
function sinc(x) {
  if (x === 0) return 1;
  return Math.sin(Math.PI * x) / (Math.PI * x);
  }

function lanczos(x, a) {
  if (x === 0) return 1;
  if (Math.abs(x) >= a) return 0;
  return a * Math.sin(Math.PI * x) * Math.sin(Math.PI * x / a) / (Math.PI * Math.PI * x * x);
}

//function to map id to row,col
function id2gridpos(id){
  let row = -1;
  let col = -1;
  if (id > globalidx) console.error("id exceeds total number of grid elements");
  else{
    row = Math.floor(id/gridSize);
    col = id % gridSize;
  }
  return [row, col]; 
}

function gridpos2id(row, col){
  return (row*gridSize + col);
}

//get children
function getchildren(agid, agList){ 
  // console.log(agid, agList);
  var origid = agid;
  while (agList[agid]['childlst'].length){
    agid = agList[agid]['childlst'][0];
  }
  if (origid == agid) return null;
  else return Math.floor(agid);
}

//get parents
function getparents(agid, agList, parents = []){
  if (agList[agid]['parentlst'].length){
    var pars = agList[agid]['parentlst'];
    agid = getparents(pars[0], agList, parents);
    agid = getparents(pars[1], agList, parents);
  }
  else{
    parents.push(agid);
  }
  return parents.filter(onlyUnique); 
}

function onlyUnique(value, index, array) {
  // console.log(array);
  return array.indexOf(value) === index;
}

function stitchmem(m1temp, m2temp, mylen, opplen){
  let m1 = m1temp.slice(m1temp.length-mylen)
  let m2 = m2temp.slice(m2temp.length-opplen)

  let mem_state = "";
  // if (m1 == "" && m2 == ""){
  //   mem_state = "";
  // }
  if (m1.length <= m2.length){
    //trim
    m2 = m2.slice(m2.length-m1.length);
    for (let i = m1.length-1; i> -1; i--){
      mem_state += m1[i];
      mem_state += m2[i]
    } 
  }
  else {
    m1 = m1.slice(m1.length-m2.length);
    for (let i = m2.length-1; i> -1; i--){
      mem_state += m1[i];
      mem_state += m2[i]
    } 
  }
  return mem_state
}

function indexOfMax(arr) {
    if (arr.length === 0) {
        return -1;
    }

    var max = arr[0];
    var maxIndex = 0;

    for (var i = 1; i < arr.length; i++) {
        if (arr[i] > max) {
            maxIndex = i;
            max = arr[i];
        }
    }

    return maxIndex;
}

const idx2act = {0: 'C', 1: 'D', 2: 'M', 3: 'S'}; 
const act2id = {'C': 0, 'D':1, 'M':2, 'S':3};
const tableidx2actmap = {};

function payoff(my_state, my_id, agList){
  if (Object.keys(agList[my_id]['policy']).indexOf(my_state) > -1){
    //if state exists, fetch act w/ the highest probability with prob 1-epsilon
    if(getRand()<=epsilon){
      let act_idx = Math.floor(getRand() * 4);
      let act = idx2act[act_idx];
      return act;
    }
    else{
      let act_idx = indexOfMax(agList[my_id]['policy'][my_state]) 
      let act = idx2act[act_idx];
      return act;
    }
  }
  else{
    if (my_state == "") console.log('error');
    //pick random act
    let act_idx = Math.floor(getRand() * 4);
    let act = idx2act[act_idx];
    //add memstate
    agList[my_id]['policy'][my_state] = getrandompolicy(); 
    return act
  }
}

var payofftable = {
            "CC": 8.0,
            "CD": 0.0,
            "CM": 8.0,
            "CS": 0.0,
            "DD": 5.0,
            "DC": 10.0,
            "DM": 10.0,
            "DS": 0.0,
            "MM": 0.0,
            "MC": 8.0,
            "MD": 0.0,
            "MS": 0.0,
            "SC": 0.0,
            "SD": 0.0,
            "SM": 0.0,
            "SS": 0.0,
        };

defaults['payofftable'] = JSON.parse(JSON.stringify(payofftable)); 

function updatePolicy(agList, use_id, c_state, act, scr, n_state){
  var act_id = act2id[act];

  var q_t = agList[use_id]['policy'][c_state][act_id];

  var qt_next = agList[use_id]['policy'][n_state] 
  if (qt_next == undefined){
    //include the new state and retry 
    agList[use_id]['policy'][n_state] = getrandompolicy(); 
    qt_next = agList[use_id]['policy'][n_state] 
  }

  var max_qt = Math.max.apply(null, qt_next);
  var q_tNew = (1-alpha)*q_t + alpha*(scr + gamma*max_qt); 

  //update
  agList[use_id]['policy'][c_state][act_id] = q_tNew;
}

function cut(arr, val){
  const index = arr.indexOf(String(val));
  if (index > -1) { // only splice array when item is found
    arr.splice(index, 1); // 2nd parameter means remove one item only
  }
  return arr;
}

function getsupernbrs(my_id, opp_id, agList){
  var my_nbrs = agList[my_id]['nbrs'];
  var opp_nbrs = agList[opp_id]['nbrs'];
  opp_nbrs = cut(opp_nbrs, my_id);
  my_nbrs = cut(my_nbrs, opp_id);

  var combined = my_nbrs.concat(opp_nbrs);
  return combined.filter(onlyUnique);
}

//play a game of ipd
function fight(availList, my_id, opp_id, agList){
  let my_mem = agList[my_id]['memlist'];
  let opp_mem = agList[opp_id]['memlist'];

  let my_state = stitchmem(my_mem, opp_mem, agList[my_id]['mem_size'], agList[opp_id]['mem_size']);
  let opp_state = stitchmem(opp_mem, my_mem, agList[opp_id]['mem_size'], agList[my_id]['mem_size']);

  let my_act = payoff(my_state, my_id, agList);
  let opp_act = payoff(opp_state, opp_id, agList); 

  let bst_idv_idx = [my_id, opp_id][indexOfMax([agList[my_id]['fitness'], agList[opp_id]['fitness']])]

  //update stress graph
  if((my_act == 'C' || my_act == 'M') && (opp_act == 'D' || opp_act == 'S')){
    //I am stressed 
    stress += 1;
  }
  //or the reverse
  if ((opp_act == 'C' || opp_act == 'M') && (my_act == 'D' || my_act == 'S')){
    //the opponent is stressed
    stress += 1;
  }
  if ((my_act == 'C' || my_act == 'M') && (opp_act == 'C' ||opp_act == 'M')){
    //decrease stress
    stress -= 1;
  }

  //update memories
  agList[my_id]['memlist'].push(my_act)
  agList[opp_id]['memlist'].push(opp_act)
  
  //update scores
  var my_score = payofftable[my_act+opp_act]
  var opp_score = payofftable[opp_act+my_act]

  agList[my_id]['fitness'] += parseFloat(my_score);
  agList[opp_id]['fitness'] += parseFloat(opp_score);

  //get next state
  let my_next_state = stitchmem(agList[my_id]['memlist'], agList[opp_id]['memlist'], agList[my_id]['mem_size'], agList[opp_id]['mem_size']);

  let opp_next_state = stitchmem(agList[opp_id]['memlist'], agList[my_id]['memlist'], agList[opp_id]['mem_size'], agList[my_id]['mem_size']);

  //update policy
  updatePolicy(agList, my_id, my_state, my_act, my_score, my_next_state) //my

  updatePolicy(agList, opp_id, opp_state, opp_act, opp_score, opp_next_state) //opp

  // console.log(my_id, opp_id, my_act, opp_act, agList);

  //physically change the board
  if(my_act == 'M' || opp_act == 'M'){
    //merge 
    var common_nbrs = getsupernbrs(my_id, opp_id, agList);  
    //create new agent
    globalidx += 1;

    agList[globalidx] = {
      parentlst : [my_id, opp_id],
      childlst : [],
      fitness: (agList[my_id]['fitness'] + agList[opp_id]['fitness'])/2, //avg.
      memlist : agList[bst_idv_idx]['memlist'], //inherit from best
      nbrs: common_nbrs, 
      policy: agList[bst_idv_idx]['policy'],
      mem_size: agList[bst_idv_idx]['mem_size'],
    }

    //add child nodes
    agList[my_id]['childlst'] = [globalidx]
    agList[opp_id]['childlst'] = [globalidx]  
  } 

  else if(my_act == 'S' || opp_act == 'S'){
    //split
    
    let mytemplen = getparents(my_id, agList, []).length;
    let opptemplen = getparents(opp_id, agList, []).length;

    if(my_act == 'S' && mytemplen >1){
       //split me 
       var pars = [];
       //remove child node
       pars = agList[my_id]['parentlst'];
       if (pars.length){
        agList[pars[0]]['childlst'] = [], //first parent
        agList[pars[1]]['childlst'] = [], //second parent

        //set scores, policy, and memory to that of the super agent
        agList[pars[0]]['fitness'] = agList[my_id]['fitness']
        agList[pars[0]]['policy'] = agList[my_id]['policy']
        agList[pars[0]]['memlist'] = agList[my_id]['memlist']

        //
        agList[pars[1]]['fitness'] = agList[my_id]['fitness']
        agList[pars[1]]['policy'] = agList[my_id]['policy']
        agList[pars[1]]['memlist'] = agList[my_id]['memlist']
        delete agList[my_id];
      }  
    }

    if(opp_act == 'S' && opptemplen > 1){
       //split me
       var pars = [] 
      //  console.log(opp_id, agList);
       //remove child node
       pars = agList[opp_id]['parentlst']
       if (pars.length){
       agList[pars[0]]['childlst'] = [], //first parent
       agList[pars[1]]['childlst'] = [], //second parent

       //set scores, policy, and memory to that of the super agent
       agList[pars[0]]['fitness'] = agList[opp_id]['fitness']
       agList[pars[0]]['policy'] = agList[opp_id]['policy']
       agList[pars[0]]['memlist'] = agList[opp_id]['memlist']

       //
       agList[pars[1]]['fitness'] = agList[opp_id]['fitness']
       agList[pars[1]]['policy'] = agList[opp_id]['policy']
       agList[pars[1]]['memlist'] = agList[opp_id]['memlist']
       delete agList[opp_id];
       }       
    }
  }  
}

function getNeighbors(idx){
  let gridinfo = id2gridpos(idx);
  var row, col;
  row = gridinfo[0];
  col = gridinfo[1];

  let neighbors = []

  if (row-1 >= 0) neighbors.push(gridpos2id(row-1, col));

  if (row+1 < gridSize) neighbors.push(gridpos2id(row+1, col));

  if (col-1 >= 0) neighbors.push(gridpos2id(row, col-1));

  if (col+1 < gridSize) neighbors.push(gridpos2id(row, col+1));

  if (row-1>=0 && col-1>=0) neighbors.push(gridpos2id(row-1, col-1));

  if (row-1>=0 && col+1<gridSize) neighbors.push(gridpos2id(row-1, col+1));

  if (row+1<gridSize && col-1>=0) neighbors.push(gridpos2id(row+1, col-1));

  if (row+1<gridSize && col+1<gridSize) neighbors.push(gridpos2id(row+1, col+1));

  return neighbors.filter(onlyUnique); 
}

//get a list of agents (ids only) involved in gameplay
function getAvailList(){
  let tempList = [];
  for (var i = 0; i<gridSize*gridSize; i++){
    var useKey = getchildren(i, agList)
    if(useKey == null) useKey = i;
    tempList.push(useKey);

    var ag_mem = getparents(useKey, agList, []);
    var ag_size = ag_mem.length >1 ? ag_mem.length:1;
    colorMap[ag_size] = spectralColor[ag_size];
  }
  return tempList.filter(onlyUnique);
}

function getrandompolicy(){
  //policy creation
  var temp_pol = [getRand(), getRand(), getRand(), getRand()];

  var pol_sum = temp_pol.reduce(function (x, y) {return x+y});
  var use_pol_prob = temp_pol.map(function(e){return e/pol_sum});

  return use_pol_prob;
}

function visualize(agList) { 
  const renderGrid = [];
  uniqueAgents = {};

  //todo: reset these variables similar to uniqueAgents 
  uniCellularFitness = [];
  multiCellularFitness = [];

  uniCooperation = [];
  multiCooperation = [];
  
  for (let y = 0; y <gridSize; y++){
    renderGrid[y] = [];
    for (let x = 0; x<gridSize; x++){
      var aidx = gridSize*y + x;
      var parmem_len = 0;
      //determine color
      var rootid = getchildren(aidx, agList) //ok
      if (rootid == null){
        parmem_len = 1; //tissue size: 1
        //unicellular tissue
        //record fitness
        uniCellularFitness.push(agList[aidx]['fitness']);
        //record cooperability (check last action)
        var uni_mem_list = agList[aidx]['memlist'];
        var u_last_act = uni_mem_list[uni_mem_list.length-1]; 
        if (u_last_act == 'C'){
          uniCooperation.push(1);
        }
        else{
          uniCooperation.push(0);
        }
      }
      else{
        //multicellular tissue
        var parmems = getparents(rootid, agList); //ok
        parmem_len = parmems.length; //gives tissue size
        //record fitness
        multiCellularFitness.push(agList[rootid]['fitness']);
        // console.log(rootid, agList[rootid]);
        //record cooperability
        var mult_mem_list = agList[rootid]['memlist'];
        var m_last_act = mult_mem_list[mult_mem_list.length-1]; 
        if (m_last_act == 'C'){
          multiCooperation.push(1);
        }
        else{
          multiCooperation.push(0);
        }

      }        
      if (parmem_len >100){
        console.log(aidx, rootid, parmem_len, parmems);
      }
      renderGrid[y][x] = colorMap[parmem_len].map(function(t) {return Math.floor(t*255)});

      uniqueAgents[parmem_len] = parmem_len; //this is stupid
      }
    }
  
  // Scale grid to canvas size
  const scale = canvasSize / gridSize;
  
  if (scale === 1) {
    // Direct 1:1 mapping when grid matches canvas
    for (let y = 0; y < gridSize; y++) {
      for (let x = 0; x < gridSize; x++) {
        const idx = (y * gridSize + x) * 4;
        const [r, g, b] = renderGrid[y][x];
        imgData.data[idx] = r;
        imgData.data[idx+1] = g;
        imgData.data[idx+2] = b;
        imgData.data[idx+3] = 255;
      }
    }
  } else {
    // Scale grid to fill canvas
    for (let y = 0; y < canvasSize; y++) {
      for (let x = 0; x < canvasSize; x++) {
        const gx = Math.floor(x / scale);
        const gy = Math.floor(y / scale);
        const idx = (y * canvasSize + x) * 4;
        const [r, g, b] = renderGrid[gy][gx];
        imgData.data[idx] = r;
        imgData.data[idx+1] = g;
        imgData.data[idx+2] = b;
        imgData.data[idx+3] = 255;
      }
    }
  }
}

//create a list of agents
// each agent is a dict carrying an id, a list of parents, a list of children, and a time-avg. payoff score

var n_colors = gridSize *gridSize; 

// agList[0]['childlst'] = [25];
// agList[1]['childlst'] = [25];
// agList[2]['childlst'] = [27];
// agList[6]['childlst'] = [27];
// agList[7]['childlst'] = [31];
// agList[8]['childlst'] = [32];
// agList[25] = {parentlst: [0, 1], childlst: [30]}
// agList[27] = {parentlst: [2, 6], childlst: [30]}
// agList[30] = {parentlst: [25, 27], childlst: [31]}
// agList[31] = {parentlst: [30, 7], childlst: [32]}
// agList[32] = {parentlst: [31, 8], childlst: []}

// console.log(getchildren(9, agList));
// console.log(getparents(31, agList));

//color settings
var spectralColor = [[0.6196, 0.0039, 0.2588], [0.6281, 0.0133, 0.2608], [0.6365, 0.0227, 0.2628], [0.645, 0.0321, 0.2648], [0.6534, 0.0414, 0.2668], [0.6619, 0.0508, 0.2688], [0.6704, 0.0602, 0.2708], [0.6788, 0.0696, 0.2728], [0.6873, 0.079, 0.2748], [0.6957, 0.0884, 0.2768], [0.7042, 0.0977, 0.2788], [0.7126, 0.1071, 0.2808], [0.7211, 0.1165, 0.2828], [0.7296, 0.1259, 0.2848], [0.738, 0.1353, 0.2868], [0.7465, 0.1446, 0.2888], [0.7549, 0.154, 0.2908], [0.7634, 0.1634, 0.2928], [0.7719, 0.1728, 0.2948], [0.7803, 0.1822, 0.2968], [0.7888, 0.1915, 0.2988], [0.7972, 0.2009, 0.3008], [0.8057, 0.2103, 0.3028], [0.8141, 0.2197, 0.3048], [0.8226, 0.2291, 0.3068], [0.8311, 0.2384, 0.3088], [0.8377, 0.2468, 0.3089], [0.8424, 0.254, 0.307], [0.8472, 0.2612, 0.3052], [0.852, 0.2684, 0.3033], [0.8567, 0.2757, 0.3015], [0.8615, 0.2829, 0.2997], [0.8663, 0.2901, 0.2978], [0.871, 0.2973, 0.296], [0.8758, 0.3046, 0.2941], [0.8806, 0.3118, 0.2923], [0.8854, 0.319, 0.2904], [0.8901, 0.3263, 0.2886], [0.8949, 0.3335, 0.2867], [0.8997, 0.3407, 0.2849], [0.9044, 0.3479, 0.283], [0.9092, 0.3552, 0.2812], [0.914, 0.3624, 0.2794], [0.9187, 0.3696, 0.2775], [0.9235, 0.3769, 0.2757], [0.9283, 0.3841, 0.2738], [0.933, 0.3913, 0.272], [0.9378, 0.3985, 0.2701], [0.9426, 0.4058, 0.2683], [0.9473, 0.413, 0.2664], [0.9521, 0.4202, 0.2646], [0.9569, 0.4275, 0.2627], [0.9582, 0.4374, 0.2674], [0.9596, 0.4474, 0.272], [0.961, 0.4574, 0.2766], [0.9624, 0.4674, 0.2812], [0.9638, 0.4774, 0.2858], [0.9652, 0.4874, 0.2904], [0.9666, 0.4974, 0.295], [0.9679, 0.5074, 0.2997], [0.9693, 0.5174, 0.3043], [0.9707, 0.5274, 0.3089], [0.9721, 0.5374, 0.3135], [0.9735, 0.5474, 0.3181], [0.9749, 0.5574, 0.3227], [0.9762, 0.5674, 0.3273], [0.9776, 0.5774, 0.3319], [0.979, 0.5874, 0.3366], [0.9804, 0.5974, 0.3412], [0.9818, 0.6074, 0.3458], [0.9832, 0.6174, 0.3504], [0.9845, 0.6274, 0.355], [0.9859, 0.6374, 0.3596], [0.9873, 0.6474, 0.3642], [0.9887, 0.6574, 0.3689], [0.9901, 0.6674, 0.3735], [0.9915, 0.6774, 0.3781], [0.9922, 0.6862, 0.3836], [0.9924, 0.6939, 0.3901], [0.9925, 0.7016, 0.3965], [0.9927, 0.7093, 0.403], [0.9928, 0.717, 0.4095], [0.993, 0.7246, 0.4159], [0.9932, 0.7323, 0.4224], [0.9933, 0.74, 0.4288], [0.9935, 0.7477, 0.4353], [0.9936, 0.7554, 0.4418], [0.9938, 0.7631, 0.4482], [0.9939, 0.7708, 0.4547], [0.9941, 0.7785, 0.4611], [0.9942, 0.7862, 0.4676], [0.9944, 0.7938, 0.474], [0.9945, 0.8015, 0.4805], [0.9947, 0.8092, 0.487], [0.9948, 0.8169, 0.4934], [0.995, 0.8246, 0.4999], [0.9952, 0.8323, 0.5063], [0.9953, 0.84, 0.5128], [0.9955, 0.8477, 0.5193], [0.9956, 0.8554, 0.5257], [0.9958, 0.8631, 0.5322], [0.9959, 0.8707, 0.5386], [0.9961, 0.8784, 0.5451], [0.9962, 0.8832, 0.5531], [0.9964, 0.888, 0.5611], [0.9965, 0.8927, 0.5691], [0.9967, 0.8975, 0.5771], [0.9968, 0.9023, 0.5851], [0.997, 0.907, 0.5931], [0.9972, 0.9118, 0.6011], [0.9973, 0.9166, 0.6091], [0.9975, 0.9213, 0.6171], [0.9976, 0.9261, 0.6251], [0.9978, 0.9309, 0.6331], [0.9979, 0.9356, 0.6411], [0.9981, 0.9404, 0.6491], [0.9982, 0.9452, 0.6571], [0.9984, 0.9499, 0.6651], [0.9985, 0.9547, 0.673], [0.9987, 0.9595, 0.681], [0.9988, 0.9642, 0.689], [0.999, 0.969, 0.697], [0.9992, 0.9738, 0.705], [0.9993, 0.9785, 0.713], [0.9995, 0.9833, 0.721], [0.9996, 0.9881, 0.729], [0.9998, 0.9928, 0.737], [0.9999, 0.9976, 0.745], [0.9981, 0.9992, 0.746], [0.9942, 0.9977, 0.74], [0.9904, 0.9962, 0.734], [0.9865, 0.9946, 0.728], [0.9827, 0.9931, 0.722], [0.9789, 0.9915, 0.716], [0.975, 0.99, 0.71], [0.9712, 0.9885, 0.704], [0.9673, 0.9869, 0.698], [0.9635, 0.9854, 0.692], [0.9596, 0.9839, 0.686], [0.9558, 0.9823, 0.68], [0.9519, 0.9808, 0.674], [0.9481, 0.9792, 0.6681], [0.9443, 0.9777, 0.6621], [0.9404, 0.9762, 0.6561], [0.9366, 0.9746, 0.6501], [0.9327, 0.9731, 0.6441], [0.9289, 0.9715, 0.6381], [0.925, 0.97, 0.6321], [0.9212, 0.9685, 0.6261], [0.9173, 0.9669, 0.6201], [0.9135, 0.9654, 0.6141], [0.9097, 0.9639, 0.6081], [0.9058, 0.9623, 0.6021], [0.902, 0.9608, 0.5961], [0.8929, 0.9571, 0.5979], [0.8838, 0.9534, 0.5998], [0.8747, 0.9497, 0.6016], [0.8657, 0.946, 0.6035], [0.8566, 0.9423, 0.6053], [0.8475, 0.9386, 0.6072], [0.8384, 0.9349, 0.609], [0.8294, 0.9313, 0.6108], [0.8203, 0.9276, 0.6127], [0.8112, 0.9239, 0.6145], [0.8022, 0.9202, 0.6164], [0.7931, 0.9165, 0.6182], [0.784, 0.9128, 0.6201], [0.7749, 0.9091, 0.6219], [0.7659, 0.9054, 0.6238], [0.7568, 0.9017, 0.6256], [0.7477, 0.898, 0.6275], [0.7386, 0.8943, 0.6293], [0.7296, 0.8907, 0.6311], [0.7205, 0.887, 0.633], [0.7114, 0.8833, 0.6348], [0.7023, 0.8796, 0.6367], [0.6933, 0.8759, 0.6385], [0.6842, 0.8722, 0.6404], [0.6751, 0.8685, 0.6422], [0.6653, 0.8646, 0.6432], [0.6547, 0.8604, 0.6434], [0.6441, 0.8563, 0.6435], [0.6334, 0.8521, 0.6437], [0.6228, 0.848, 0.6438], [0.6122, 0.8438, 0.644], [0.6016, 0.8397, 0.6441], [0.591, 0.8355, 0.6443], [0.5804, 0.8314, 0.6444], [0.5698, 0.8272, 0.6446], [0.5592, 0.8231, 0.6448], [0.5486, 0.8189, 0.6449], [0.5379, 0.8148, 0.6451], [0.5273, 0.8106, 0.6452], [0.5167, 0.8065, 0.6454], [0.5061, 0.8023, 0.6455], [0.4955, 0.7982, 0.6457], [0.4849, 0.794, 0.6458], [0.4743, 0.7899, 0.646], [0.4637, 0.7857, 0.6461], [0.4531, 0.7815, 0.6463], [0.4424, 0.7774, 0.6464], [0.4318, 0.7732, 0.6466], [0.4212, 0.7691, 0.6468], [0.4106, 0.7649, 0.6469], [0.4, 0.7608, 0.6471], [0.392, 0.7519, 0.6507], [0.384, 0.7429, 0.6544], [0.376, 0.734, 0.6581], [0.368, 0.7251, 0.6618], [0.36, 0.7162, 0.6655], [0.352, 0.7073, 0.6692], [0.344, 0.6983, 0.6729], [0.336, 0.6894, 0.6766], [0.328, 0.6805, 0.6803], [0.32, 0.6716, 0.684], [0.312, 0.6627, 0.6877], [0.304, 0.6537, 0.6913], [0.296, 0.6448, 0.695], [0.288, 0.6359, 0.6987], [0.28, 0.627, 0.7024], [0.272, 0.6181, 0.7061], [0.2641, 0.6092, 0.7098], [0.2561, 0.6002, 0.7135], [0.2481, 0.5913, 0.7172], [0.2401, 0.5824, 0.7209], [0.2321, 0.5735, 0.7246], [0.2241, 0.5646, 0.7283], [0.2161, 0.5556, 0.7319], [0.2081, 0.5467, 0.7356], [0.2001, 0.5378, 0.7393], [0.1995, 0.529, 0.7391], [0.2062, 0.5202, 0.7349], [0.213, 0.5114, 0.7308], [0.2198, 0.5027, 0.7266], [0.2265, 0.4939, 0.7225], [0.2333, 0.4851, 0.7183], [0.2401, 0.4764, 0.7142], [0.2468, 0.4676, 0.71], [0.2536, 0.4588, 0.7059], [0.2604, 0.4501, 0.7017], [0.2671, 0.4413, 0.6976], [0.2739, 0.4325, 0.6934], [0.2807, 0.4238, 0.6893], [0.2874, 0.415, 0.6851], [0.2942, 0.4062, 0.681], [0.301, 0.3975, 0.6768], [0.3077, 0.3887, 0.6727], [0.3145, 0.3799, 0.6685], [0.3213, 0.3712, 0.6644], [0.328, 0.3624, 0.6602], [0.3348, 0.3536, 0.6561], [0.3416, 0.3449, 0.6519], [0.3483, 0.3361, 0.6478], [0.3551, 0.3273, 0.6436], [0.3619, 0.3186, 0.6394], [0.3686, 0.3098, 0.6353]];
//shuffle colors
function shuffle(array) {
  let currentIndex = array.length;

  // While there remain elements to shuffle...
  while (currentIndex != 0) {

    // Pick a remaining element...
    let randomIndex = Math.floor(getRand() * currentIndex);
    currentIndex--;

    // And swap it with the current element.
    [array[currentIndex], array[randomIndex]] = [
      array[randomIndex], array[currentIndex]];
  }
}
shuffle(spectralColor);

function trace_to_root(tar_id, tr_list, agList){
  var traced_list = [];
  for (var i = 0; i<tr_list.length; i++){
    var rollId = tr_list[i];
    var root_trace_id = getchildren(rollId, agList); //ok
    if (root_trace_id == null){
      root_trace_id = rollId;
    }
    if (root_trace_id == tar_id){continue;} 
    traced_list.push(root_trace_id); 
  }
  return traced_list.filter(onlyUnique);
} 

function singleGame(){
  if (breakFlag == true){
    n = totRounds;
    return;
  }
  let availList = getAvailList() //ok

  //visualize previous game
  visualize(agList); //ok

  //pick an agent 
  var my_id = Math.floor(availList[Math.floor(getRand() * availList.length)]) //ok

  //pick one of its neighbors
  var opp_choices = agList[my_id]['nbrs']; //ok
  // trace each neighbor to its root
  var actual_opp_choices = trace_to_root(my_id, opp_choices, agList);
  // console.log(availList.length);
  // console.log(availList, my_id, actual_opp_choices);

  //if no neighbors exist, exit
  if (actual_opp_choices.length){
    var opp_id = Math.floor(actual_opp_choices[Math.floor(getRand() * actual_opp_choices.length)]) 
  }
  else {
    breakFlag=true;  
    console.log("singular agent at: ", n, "ts", "w/", "memory limit of: ", MAX_MEM);
    return;
  } 

  //play a game
  fight(availList, my_id, opp_id, agList); 

  ++n;
}

function saveState(){
  //save game state
  saveBuffer[n] = JSON.parse(JSON.stringify(agList));
}

function renderSpecificFrame(fr_num){
  var use_keys = Object.keys(saveBuffer); 
  if (use_keys.find(function (x){return x==fr_num}) == undefined){
    fr_num = use_keys[use_keys.length-1]; 
  }
  // console.log(saveBuffer, gridSize, MAX_MEM);
  visualize(saveBuffer[fr_num]);
    
  colorbar();
  update_graph_values(fr_num);

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.putImageData(imgData, 0, 0);
  
  if(n<totRounds && n >1){
    infotext.textContent = "Paused | \u23EA Game: " + String(fr_num);
  }
  else if (n == totRounds){
    infotext.textContent = "Finished | \u23EA Game: " + String(fr_num);
  }
}

function animate(){
  //buffer update
  saveState();
  
  colorbar();
  update_graph_values();
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.putImageData(imgData, 0, 0);

  singleGame(); 

  //slider progress
  const trail_length = Object.keys(saveBuffer).length; 
  if (trail_length > slider.max){
    slider.max = trail_length+1; 
  }
  slider.value = trail_length; 

  if (n<totRounds && playicon.hidden){
    infotext.textContent = "Running | \u25B6  Game : " + String(n);
    infotext.style.color = 'black';
    requestAnimationFrame(animate);
  }  
  else if (n>=totRounds){
    infotext.textContent = "Finished (Single Agent) | Game : " + String(Object.keys(saveBuffer)[Object.keys(saveBuffer).length-1]);
  }
  else if (n >2 && pauseicon.hidden){
    infotext.textContent =  'Paused | \u23F8 Game : ' + String(n-1); 
  }  
}

initializeGrid();
renderSpecificFrame(0);

slider.addEventListener('input', ()=> {
  const progress = parseFloat(slider.value);
  const bufferLen = Object.keys(saveBuffer).length;
  //pause on touch
  if (breakFlag == false){
    pauseicon.hidden = true;
    playicon.hidden=false;
  }
  //prevent moving beyond curr timestep
  if (progress <= bufferLen){ 
    //render frame
    renderSpecificFrame(progress)
  }
  else{
    slider.value = bufferLen;
  }
})

playicon.addEventListener('click', ()=>{
  playicon.hidden=true;
  //start animation
  if (breakFlag == false){
    requestAnimationFrame(animate);
    //show pause icon
  }
  pauseicon.hidden=false;
  reseticon.hidden=false;
  //update slider
  slider.max = 100.0;
  // add slider functionality
  })

pauseicon.addEventListener('click', ()=>{
  if (breakFlag == false){
    pauseicon.hidden=true;
    playicon.hidden=false;
  }
  //pause animation
})

reseticon.addEventListener('click', () => {
  playicon.hidden=false;
  pauseicon.hidden=true;

  // //replay buffer
  // n = 0;
  // colorMap = {};
  // maxLimit = 0.0;
  // saveBuffer = {};
  // globalidx = gridSize*gridSize;
  // breakFlag = false;
  // slider.value = 0.0;
  initializeGrid();
  update_graph_values(0); //clear graph

  //clear existing canvas
  infotext.textContent = "Click \u25B6 to Start"
  renderSpecificFrame(0);
  reseticon.hidden = true;
})

// drop down button
const dotsbtn = document.getElementById('dotsBtn');
playbackctrl.appendChild(dotsbtn);

//settings panel
const form = document.createElement('form');
form.id = 'form';

//grid size setting
const gridsizelbl = document.createElement('h4');
gridsizelbl.className = "simsettingslabel";
gridsizelbl.textContent = "Simulation Settings"

const labelsz = document.createElement('label');
labelsz.setAttribute('for', 'gridsz');
labelsz.textContent = "1) Number of Agents ";

const gridsz = document.createElement('input');
gridsz.type = 'range';
gridsz.id = "gridsz";
gridsz.min = 2;
gridsz.max = canvasSize;
gridsz.step = 1;
gridsz.value = 10;

const valueDisplay = document.createElement('span');
valueDisplay.id = 'gridszvalue';
valueDisplay.textContent = gridsz.value + "x" + gridsz.value + " (" + Math.pow(gridsz.value, 2) + ")";

gridsz.oninput = function(){
  valueDisplay.textContent = this.value + "x" + this.value + " (" + Math.pow(this.value, 2) + ")";
};

form.appendChild(gridsizelbl);
form.appendChild(document.createElement('br'));
form.appendChild(labelsz);
form.appendChild(gridsz);
form.appendChild(valueDisplay);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));

//mem len setting
const labelmem = document.createElement('label');
labelmem.setAttribute('for', 'memlen');
labelmem.textContent = "2) Max. Agent Memory";

const memlen = document.createElement('input');
memlen.type = 'range';
memlen.id = "memlen";
memlen.min = 1;
memlen.max = 5;
memlen.step = 1;
memlen.value = 5;

const valueDisplayMem = document.createElement('span');
valueDisplayMem.id = 'memlenvalue';
valueDisplayMem.textContent = memlen.value;

memlen.oninput = function(){
  valueDisplayMem.textContent = this.value;
};

form.appendChild(labelmem);
form.appendChild(memlen);
form.appendChild(valueDisplayMem);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));

//Q-learning settings
//label settings
const qlearninglabel = document.createElement('h4');
qlearninglabel.className = "qlearninglabel";
qlearninglabel.textContent = "Q-learning Setting"

//alpha
const labelrl_alpha = document.createElement('label');
labelrl_alpha.setAttribute('for', 'alpha');
labelrl_alpha.textContent = "3) learning-rate, \u03B1";

const rl_alpha = document.createElement('input');
rl_alpha.type = 'range';
rl_alpha.id = "rl_alpha";
rl_alpha.min = 0.05;
rl_alpha.max = 0.99;
rl_alpha.step = 0.01;
rl_alpha.value = 0.2;

const valueDisplayrl_alpha = document.createElement('span');
valueDisplayrl_alpha.id = 'rl_alphavalue';
valueDisplayrl_alpha.textContent = rl_alpha.value;

rl_alpha.oninput = function(){
  valueDisplayrl_alpha.textContent = this.value;
};

form.appendChild(qlearninglabel);
form.appendChild(document.createElement('br'));
form.appendChild(labelrl_alpha);
form.appendChild(rl_alpha);
form.appendChild(valueDisplayrl_alpha);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));

//gamma
const labelrl_gamma = document.createElement('label');
labelrl_gamma.setAttribute('for', 'gamma');
labelrl_gamma.textContent = "4) Discount-factor, \u03B3";

const rl_gamma = document.createElement('input');
rl_gamma.type = 'range';
rl_gamma.id = "rl_gamma";
rl_gamma.min = 0.05;
rl_gamma.max = 1.0;
rl_gamma.step = 0.05;
rl_gamma.value = 0.95;

const valueDisplayrl_gamma = document.createElement('span');
valueDisplayrl_gamma.id = 'rl_gammavalue';
valueDisplayrl_gamma.textContent = rl_gamma.value;

rl_gamma.oninput = function(){
  valueDisplayrl_gamma.textContent = this.value;
};

form.appendChild(labelrl_gamma);
form.appendChild(rl_gamma);
form.appendChild(valueDisplayrl_gamma);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));

//epsilon
const labelrl_epsilon = document.createElement('label');
labelrl_epsilon.setAttribute('for', 'epsilon');
labelrl_epsilon.textContent = "5) Exploration Prob., \u03B5";

const rl_epsilon = document.createElement('input');
rl_epsilon.type = 'range';
rl_epsilon.id = "rl_epsilon";
rl_epsilon.min = 0.00;
rl_epsilon.max = 1.0;
rl_epsilon.step = 0.1;
rl_epsilon.value = 0.1;

const valueDisplayrl_epsilon = document.createElement('span');
valueDisplayrl_epsilon.id = 'rl_epsilonvalue';
valueDisplayrl_epsilon.textContent = rl_epsilon.value;

rl_epsilon.oninput = function(){
  valueDisplayrl_epsilon.textContent = this.value;
};

form.appendChild(labelrl_epsilon);
form.appendChild(rl_epsilon);
form.appendChild(valueDisplayrl_epsilon);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));

//payoff table
//label
const paylabel = document.createElement('h4');
paylabel.className = "payofftablelabel";
paylabel.textContent = "Pay-off Table Settings"

const table = document.createElement('table');
table.id = "payofftable";
const tbody = document.createElement('tbody');
tbody.id = "tbody";
//table headers
//p1
const player1col = document.createElement('tr');
const colheader = document.createElement('th');
colheader.id = "player1text";
colheader.scope = "col";
colheader.textContent = "Player 1";
colheader.rowSpan = 7;
colheader.style.border = "none";
player1col.appendChild(colheader);
tbody.appendChild(player1col);

//p2
const player2row = document.createElement('tr');
const rowheader = document.createElement('th');
rowheader.textContent = "Player 2";
rowheader.colSpan = 4;
rowheader.style.border = "none"
player2row.appendChild(rowheader);
tbody.appendChild(player2row);

//main table
for (let i = 0; i<5; i++){
  const row = document.createElement('tr');
  for (let j = 0; j<5; j++){
    //row 0 labels
    if(i==0 && j>0){
      const cell = document.createElement('td');
      cell.textContent = idx2act[j-1]; 
      cell.style.paddingBottom = "5px"; 
      cell.style.border = "none";
      row.appendChild(cell);
      tbody.appendChild(row);
      continue;
    }
    //col0 labels
    else if(i>0 && j==0){
      const cell = document.createElement('td');
      cell.textContent = idx2act[i-1]; 
      cell.style.paddingRight = "5px"; 
      cell.style.border = "none";
      row.appendChild(cell);
      tbody.appendChild(row);
      continue;
    }
    if(i==0 && j==0){
      const cell = document.createElement('td');
      cell.textContent = "";
      cell.style.border = "none";
      row.appendChild(cell);
      tbody.appendChild(row);
      continue
    }
    const cell = document.createElement('td');
    const cinp = document.createElement('input')
    cinp.type = "number";
    cinp.step = "any";
    cinp.style.width = "30px"; 
    cinp.style.height = "10px"; 
    cinp.style.textAlign = "center";
    cinp.value = parseFloat(payofftable[`${idx2act[i-1]}${idx2act[j-1]}`]);
    cell.appendChild(cinp);
    row.appendChild(cell);
  }
  tbody.appendChild(row);
}
table.appendChild(tbody);
// const caption = table.createCaption();
// caption.textContent = " ";
table.setAttribute("border", "1");

form.appendChild(paylabel);
form.appendChild(document.createElement('br'));
form.appendChild(table);
form.appendChild(document.createElement('br'));
form.appendChild(document.createElement('br'));


const submit = document.createElement('input');
submit.id = "submit";
submit.type = 'submit';
submit.value = 'Apply';
form.appendChild(submit);

const reset = document.createElement('button');
reset.id = "resetbtn";
reset.type = "reset";
reset.textContent = "Reset";
form.appendChild(reset);

playbackctrl.appendChild(form);
form.style.display = 'none'; 

dotsbtn.addEventListener('click', function(event){
  form.style.display = form.style.display === 'block' ? 'none' : 'block';
});

form.onsubmit = function (event) {
  event.preventDefault();

  //pause first
  if (breakFlag == false){
    pauseicon.hidden=true;
    playicon.hidden=false;
  }
  gridSize = parseInt(document.getElementById('gridsz').value);
  MAX_MEM = parseInt(document.getElementById('memlen').value);
  //rl settings
  //alpha
  alpha = parseFloat(document.getElementById('rl_alpha').value);

  //gamma
  gamma = parseFloat(document.getElementById('rl_gamma').value);

  //epsilon
  epsilon = parseFloat(document.getElementById('rl_epsilon').value);

  // update payoff table 
  for(let ridx =3; ridx<7; ridx++){
    const p1act = idx2act[ridx-3];
    for (let cidx = 1; cidx<5; cidx++){
      //read table values, 
      //set p1 and p2's scores
      const p2act = idx2act[cidx-1];
      const cval = table.rows[ridx].cells[cidx].querySelector('input').value;
      payofftable[`${p1act}${p2act}`] = cval; //p1, p2 payoff 
    }
  }
  console.log(gridSize, MAX_MEM, alpha, gamma, epsilon, payofftable);
  
  initializeGrid();
  
  // Canvas always stays at 600x600, grid size determines pixel density
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  // Canvas dimensions stay fixed
  canvas.width = canvasSize;
  canvas.height = canvasSize;

  //resize colorbar
  if(gridSize <6){
    cbar_box_width = 15;
    cbar_box_height = 4;
    box_spacing = 5;
    width_multipler = 45;
  }
  else if(gridSize < 50){
    cbar_box_width = 20;
    cbar_box_height = 10;
    box_spacing = 5;
    width_multipler = 45;
  }
  else{
    // For very large grids, make colorbar more compact
    cbar_box_width = 15;
    cbar_box_height = 8;
    box_spacing = 3;
    width_multipler = 35;
  }

  colorbar();
  update_graph_values(0); //clear graph

  //clear existing canvas
  imgData = ctx.createImageData(canvasSize, canvasSize);
  ctx.putImageData(imgData, 0, 0);
  infotext.textContent = "Click \u25B6 to Start"
  renderSpecificFrame(0);
  reseticon.hidden = true;
  // this.style.display = 'none';
}

form.addEventListener('reset', function(event){
  event.preventDefault();
  //reset to default
  form.gridsz.value = defaults['n_agents'];
  document.getElementById('gridszvalue').textContent = defaults['n_agents'] + "x" + defaults['n_agents'] + " (" + Math.pow(defaults['n_agents'], 2) + ")";

  form.memlen.value = defaults['memory'];
  document.getElementById('memlenvalue').textContent = defaults['memory'];

  form.rl_alpha.value = defaults['alpha'];
  document.getElementById('rl_alphavalue').textContent = defaults['alpha'];

  form.rl_gamma.value = defaults['gamma'];
  document.getElementById('rl_gammavalue').textContent = defaults['gamma'];

  form.rl_epsilon.value = defaults['epsilon'];
  document.getElementById('rl_epsilonvalue').textContent = defaults['epsilon'];

  const tbdy = document.getElementById('tbody')
  for (let i = 0; i<7; i++){
    const row = tbdy.rows[i];
    for (let j = 0; j<5; j++){
      const cell = row.cells[j];
      if (i>=3  && j >=1){
        cell.querySelector('input').value = parseFloat(defaults['payofftable'][`${idx2act[i-3]}${idx2act[j-1]}`]);
      }
    }
  }
  //     if(i==0 && j>0){
  //       const cell = document.createElement('td');
  //       cell.textContent = idx2act[j-1]; 
  //       cell.style.paddingBottom = "5px"; 
  //       cell.style.border = "none";
  //       row.appendChild(cell);
  //       tbody.appendChild(row);
  //       continue;
  //     }
  //     //col0 labels
  //     else if(i>0 && j==0){
  //       const cell = document.createElement('td');
  //       cell.textContent = idx2act[i-1]; 
  //       cell.style.paddingRight = "5px"; 
  //       cell.style.border = "none";
  //       row.appendChild(cell);
  //       tbody.appendChild(row);
  //       continue;
  //     }
  //     if(i==0 && j==0){
  //       const cell = document.createElement('td');
  //       cell.textContent = "";
  //       cell.style.border = "none";
  //       row.appendChild(cell);
  //       tbody.appendChild(row);
  //       continue
  //     }
  //     const cell = document.createElement('td');
  //     const cinp = document.createElement('input')
  //     cinp.type = "number";
  //     cinp.step = "any";
  //     cinp.style.width = "30px"; 
  //     cinp.style.height = "10px"; 
  //     cinp.style.textAlign = "center";
  //     cinp.value = parseFloat(payofftable[`${idx2act[i-1]}${idx2act[j-1]}`]);
  //     cell.appendChild(cinp);
  //     row.appendChild(cell);
  //   }
  //   tbody.appendChild(row);
  // }

});
