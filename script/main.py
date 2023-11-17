import gpxpy
import gpxpy.gpx
import folium
import matplotlib.pyplot as plt 
import csv 

# pip install folium
  
x = [] 
y = [] 
  
with open('profile.csv','r') as csvfile: 
    lines = csv.reader(csvfile, delimiter=',') 
    for row in lines: 
        x.append(row[0]) 
        y.append(int(row[1])) 
  
plt.plot([int(i) for i in x], y)
plt.xticks(range(int(x[0]),int(x[-1]),240))
plt.xlabel('Time') 
plt.ylabel('alt')  
plt.show() 



m = folium.Map()

f = open("flight.json","r")

folium.GeoJson(f.read()).add_to(m)

m.save("index.html")


gpx = gpxpy.gpx.GPX()
gpx_track = gpxpy.gpx.GPXTrack()
gpx.tracks.append(gpx_track)
gpx_segment = gpxpy.gpx.GPXTrackSegment()
gpx_track.segments.append(gpx_segment)

f = open("trace","r")
lines = f.readlines()

for line in lines:
    latlng = line.split(',')
    
    gpx_segment.points.append(gpxpy.gpx.GPXTrackPoint(float(latlng[0]), float(latlng[1])))



gpx_file = open("test.gpx", "w")
gpx_file.write(gpx.to_xml())
gpx_file.close()