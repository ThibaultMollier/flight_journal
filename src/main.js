const { invoke } = window.__TAURI__.tauri;

// Leaflet map

var map = L.map('map').setView([45.5, 6.2], 10);
let trace = L.geoJSON().addTo(map);
let marker = L.marker([0,0]).addTo(map);

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
  marker.setLatLng([0, 0]);
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

  let profile = new Profile();
  profile.draw(flight.profile);
  profile.listen((lat,lng) => {
    marker.setLatLng([lat, lng])
  },
  (wheelDelta) => {
    let zoom = map.getZoom();
    if(wheelDelta < 0)
    {
      zoom -= 1
    }else{
      zoom += 1
    }
    map.setView([marker._latlng.lat,marker._latlng.lng],zoom);
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

