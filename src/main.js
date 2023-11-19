const { invoke } = window.__TAURI__.tauri;

// Leaflet map

var map = L.map('map').setView([45.5, 6.2], 10);
let trace = L.geoJSON().addTo(map);

// L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
//     maxZoom: 19,
//     attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
// }).addTo(map);

L.tileLayer('https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: 'Map data: © OpenStreetMap contributors, SRTM | Map style: © OpenTopoMap (CC-BY-SA)'
}).addTo(map);


function load_flght(flight)
{
  trace.remove();
  let geojson = JSON.parse(flight.track);
  trace = L.geoJSON(geojson, {
    style: function(feature) {
        switch (feature.id) {
            case 'seg_in':
            case 'seg_out':
            case 'seg0':
            case 'seg1': 
            case 'seg2': return {color: "#ff7800",opacity: 0.7};
            case 'closing': return {opacity: 0};
            case 'flight': return {color: "#3A00E5"};
        }
    },
    pointToLayer: function (feature, latlng) {
      switch (feature.id) {
        case 'ep_finish':
        case 'ep_start':
        case 'tp0':
        case 'tp1': 
        case 'tp2': return L.circleMarker(latlng,{
          radius: 5,
          fillColor: "#ffffff",
          fillOpacity:1,
          color: "#000",
          weight: 1,
        });
        case 'land0':
        case 'launch0': return L.marker(latlng);
      }
    }
  });
  trace.addTo(map);
  map.fitBounds(trace.getBounds());
  // map.setView([geojson.features[9].geometry.coordinates[1],geojson.features[9].geometry.coordinates[0]], 11);
  console.log(geojson);

  let data = CSVToArray(flight.profile)

  let graph = document.getElementById("graph_container");
  let curve = document.getElementById("curve");
  let xstep = graph.offsetWidth / data[1].length;
  let ystep = graph.offsetHeight / (Math.ceil(Math.max(...data[1])/1000)*1000);
  let curve_str = "M0.0,"+(graph.offsetHeight - data[1][0]*ystep);
  
  console.log(Math.max(...data[1]));
  console.log(Math.ceil(Math.max(...data[1])/1000)*1000)

  for (let index = 1; index < (data[1].length-1); index++) {
    curve_str += "L" + index*xstep + "," + (graph.offsetHeight - data[1][index]*ystep);
  }

  curve.setAttribute("d",curve_str);
  let axes = document.getElementById("axes");
  let axe_str = "";

  for (let alt = 1000; alt < Math.max(...data[1]); alt+=1000) {
    axe_str += "M0," + (graph.offsetHeight -alt*ystep) +"H"+graph.offsetWidth ;
  }
  axes.setAttribute("d",axe_str);
  

  graph.addEventListener("mousemove", (event) => {
    let i = Math.trunc(event.offsetX/xstep);
    console.log(data[1][i]);
    let cursor = document.getElementById("cursor");
    cursor.setAttribute("d","M" + event.offsetX + ",0V" + graph.offsetHeight);

    let text = document.getElementById("text");
    text.setAttribute("x",event.offsetX);
    text.setAttribute("y",(graph.offsetHeight - data[1][i]*ystep + 5));
    text.textContent = data[1][i] +"m";
    // cursor.parentNode.appendChild(text);

    // document.getElementById("text").textContent = data[1][i] +"m";
  });
 
}

// Tree list

invoke('history').then((history) => build_tree(history))


function format_duration(duration)
{
  let duration_str = "";
  let hour = Math.trunc(duration/60);
  if (hour != 0) 
  {
    duration_str += hour +"h"
  }
  let min = Math.trunc(duration%60);
  if (min < 10)
  {
    duration_str += "0"
  }
  duration_str += min +"min"
  duration_str = duration_str.padEnd(8,'\u2000');
  return duration_str;
}

function format_distance(score,type)
{
  let multiplier = 1000;

  if (type == '"tri"')
  {
    multiplier = 1200;
  }else if(type == '"fai"')
  {
    multiplier = 1400;
  }

  let distance_str = (score/multiplier).toFixed(1) + "km";
  distance_str = distance_str.padEnd(7,'\u2000');
  return distance_str;
}

function build_tree(flightlist) {
  let tree = document.getElementById("tree");

  for (const key in flightlist) {
    if (Object.hasOwnProperty.call(flightlist, key)) {
      const element = flightlist[key];

      let list_element = document.createElement("li");

      list_element.addEventListener('click',flight_select,false);
      list_element.flight_id = element.flight_id;

      list_element.innerHTML = "<img src=\"assets/Icons_calendar.svg\"><div>"+ element.date +"</div>";
      list_element.innerHTML += "<img src=\"assets/Icons_clock.svg\"><div>"+ format_duration(element.duration)+"</div>";
      if(element.code == '"tri"' || element.code == '"fai"')
      {
        list_element.innerHTML += "<img src=\"assets/Icons_tri.svg\">"
      }else
      {
        list_element.innerHTML += "<img src=\"assets/Icons_op.svg\">"
      }
      list_element.innerHTML += "<div>"+ format_distance(element.score,element.code)+"</div>";

      tree.append(list_element);
      
    }
  }
}

function flight_select(evt) {
  let prev = document.getElementById('selected');
  if (prev != null){
    prev.removeAttribute('id')
  }
  evt.currentTarget.setAttribute('id','selected');
  invoke('select', {id:parseInt(evt.currentTarget.flight_id)}).then((flight) => load_flght(flight));
}

window.addEventListener("DOMContentLoaded", () => {

});



function CSVToArray( strData, strDelimiter ){
  // Check to see if the delimiter is defined. If not,
  // then default to comma.
  strDelimiter = (strDelimiter || ",");

  // Create a regular expression to parse the CSV values.
  var objPattern = new RegExp(
    (
      // Delimiters.
      "(\\" + strDelimiter + "|\\r?\\n|\\r|^)" +

      // Quoted fields.
      "(?:\"([^\"]*(?:\"\"[^\"]*)*)\"|" +

      // Standard fields.
      "([^\"\\" + strDelimiter + "\\r\\n]*))"
    ),
    "gi"
    );


  // Create an array to hold our data. Give the array
  // a default empty first row.
  var arrData = [[]];

  // Create an array to hold our individual pattern
  // matching groups.
  var arrMatches = null;


  // Keep looping over the regular expression matches
  // until we can no longer find a match.
  while (arrMatches = objPattern.exec( strData )){

    // Get the delimiter that was found.
    var strMatchedDelimiter = arrMatches[ 1 ];

    // Check to see if the given delimiter has a length
    // (is not the start of string) and if it matches
    // field delimiter. If id does not, then we know
    // that this delimiter is a row delimiter.
    if (
      strMatchedDelimiter.length &&
      (strMatchedDelimiter != strDelimiter)
      ){

      // Since we have reached a new row of data,
      // add an empty row to our data array.
      arrData.push( [] );

    }


    // Now that we have our delimiter out of the way,
    // let's check to see which kind of value we
    // captured (quoted or unquoted).
    if (arrMatches[ 2 ]){

      // We found a quoted value. When we capture
      // this value, unescape any double quotes.
      var strMatchedValue = arrMatches[ 2 ].replace(
        new RegExp( "\"\"", "g" ),
        "\""
        );

    } else {

      // We found a non-quoted value.
      var strMatchedValue = arrMatches[ 3 ];

    }


    // Now that we have our value string, let's add
    // it to the data array.
    arrData[ arrData.length - 1 ].push( strMatchedValue );
  }

  // Return the parsed data.
  return( arrData );
}
