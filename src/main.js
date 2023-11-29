const { invoke } = window.__TAURI__.tauri;

// Leaflet map

var map = L.map('map').setView([45.5, 6.2], 10);
let trace = L.geoJSON().addTo(map);
// let marker = L.marker([0,0]).addTo(map);
let marker = L.circleMarker([0,0],{
  radius: 6,
  fillColor: "#3A00E5",
  fillOpacity:1,
  color: "white",
  weight: 1,
  opacity:0.5,
}).addTo(map);

// L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
//     maxZoom: 19,
//     attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
// }).addTo(map);

L.tileLayer('https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png', {
    maxZoom: 19,
    attribution: 'Map data: © OpenStreetMap contributors, SRTM | Map style: © OpenTopoMap (CC-BY-SA)'
}).addTo(map);


function load_flight(flight)
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
  map.fitBounds(trace.getBounds(),{paddingBottomRight: [0,200]});
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
//let tree = new Tree();
// invoke('history').then((history) => tree.build_tree(history,flight_select)).catch((error) => alert(error));

let flightlist = new FlightList(load_flight);

/*function flight_select(evt) {
  let prev = document.getElementById('selected');
  if (prev != null){
      prev.removeAttribute('id')
  }
  evt.currentTarget.setAttribute('id','selected');
  invoke('select', {id:parseInt(evt.currentTarget.flight_id)}).then((flight) => load_flght(flight)).catch((error) => alert(error));;
}*/

window.addEventListener("DOMContentLoaded", () => {

});

