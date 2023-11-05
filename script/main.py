import gpxpy
import gpxpy.gpx



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