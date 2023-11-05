use crate::flight_manager::flight_data::FlightPoint;
use chrono::{Utc, TimeZone, Datelike};
use geoutils::Location;

const EPSILON_COEF:f64 = 1.5;

pub struct FlightSimplify{
}


impl FlightSimplify {
    pub fn simplify(pointlist: &Vec<FlightPoint>,n: u32) -> Vec<FlightPoint>
    {
        let mut dmax: f64 = 0.0;
        let mut index: u32 = 0;
        let mut cpt: u32 = 1;
        let end: usize = pointlist.len();
        let mut result: Vec<FlightPoint> = Vec::new();

        if end < 3
        {
            return pointlist.to_vec();
        }

        for pt in pointlist.get(1..end).unwrap()
        {
            let d1: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            let d2: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist.last().unwrap().lat, pointlist.last().unwrap().long)).unwrap().meters();

            if (d1 + d2) > dmax
            {
                dmax = d1 + d2;
                index = cpt;
            }
            cpt += 1;
        }

        if n > 0 && index > 1
        {
            let n = n - 1;

            let res1 = Self::simplify(&pointlist.get(..index as usize).unwrap().to_vec(), n);
            let mut res2 = Self::simplify(&pointlist.get(index as usize..).unwrap().to_vec(), n);

            result = res1;
            result.append(&mut res2);
        }else {
            result.push(pointlist[0].clone());
            result.push(pointlist.last().unwrap().clone());
        }

        return result;
    }

    fn distance_between(p1:&FlightPoint, p2:&FlightPoint) -> f32
    {
        let d1: f32 = p1.lat - p2.lat;
        let d2: f32 = p1.long - p2.long;

        let d: f32 = d1*d1 + d2*d2;

        return f32::sqrt(d);
    }

    pub fn triangle(pointlist: &Vec<FlightPoint>) -> (Vec<FlightPoint>,f64)
    {
        // let step = 10;
        let mut index: usize = 0;
        let mut dmax: f64 = 0.0;
        let mut p1:FlightPoint = FlightPoint { 
            time: Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap(),
            lat: 0.0,
            long: 0.0, 
            alt: 0, 
            alt_gps: 0 
        };
        let mut p2:FlightPoint = p1.clone();
        let mut p3:FlightPoint = p1.clone();

        for pt1 in pointlist
        {
            index += 1;
            if index < (pointlist.len() - 1)
            {
                for pt2 in pointlist.get(index..).unwrap()
                {
                    let loc1 = Location::new(pt1.lat, pt1.long);
                    let loc2 = Location::new(pt2.lat, pt2.long);
                    let d1: f64 = loc1.distance_to(&loc2).unwrap().meters();

                    // let d1 = FlightData::distance_between(pt1, pt2) as f64;

                    if d1 > dmax/4.0 
                    {
                        for pt3 in pointlist.get(index + 1..).unwrap()
                        {
                            let loc3 = Location::new(pt3.lat, pt3.long);
                            let d2: f64 = loc1.distance_to(&loc3).unwrap().meters();
                            let d3: f64 = loc2.distance_to(&loc3).unwrap().meters();

                            // let d2 = FlightData::distance_between(pt1, pt3) as f64;
                            // let d3 = FlightData::distance_between(pt2, pt3) as f64;

                            if (d1 + d2 + d3) > dmax
                            {
                                dmax = d1 + d2 + d3;
                                p1 = pt1.clone();
                                p2 = pt2.clone();
                                p3 = pt3.clone();
                                // println!("{:.1}",dmax);
                            }
                        }
                    }
                }
            }
        }

        // dbg!(dmax);
        return (vec![p1,p2,p3],dmax);
    }

    fn compute_epsilon(pointlist: &Vec<FlightPoint>) -> f64
    {
        let mut dmax: f64 = 0.0;
        let end: usize = pointlist.len();

        for pt in pointlist.get(1..end).unwrap()
        {
            let d1: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            let d2: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[end - 1].lat, pointlist[end - 1].long)).unwrap().meters();

            if (d1 + d2) > dmax
            {
                dmax = d1 + d2;
            }
        }

        return dmax/EPSILON_COEF;
    }

    fn magnitude(p1: &FlightPoint, p2: &FlightPoint) -> f32
    {
        let line = FlightPoint{
            lat: p2.lat - p1.lat,
            long: p2.long - p1.long,
            time: p2.time,
            alt: 0,
            alt_gps: 0,
        };

        return f32::sqrt(line.lat*line.lat + line.long*line.long);
    }

    fn distance_point_line(p: &FlightPoint, line_start: &FlightPoint, line_end: &FlightPoint) -> f32
    {
        let linemag = Self::magnitude(&line_start, &line_end);

        let u = (((p.lat - line_start.lat) * (line_end.lat - line_start.lat)) +
            ((p.long - line_start.long) * (line_end.long - line_start.long)))/
            (linemag*linemag);

        // if (u < 0.0) || (u > 1.0)
        // {
        //     // dbg!(u);
        //     return 0.0;
        // }

        let intersection = FlightPoint
        {
            lat: line_start.lat + u * (line_end.lat - line_start.lat),
            long: line_start.long + u * (line_end.long - line_start.long),
            time: line_start.time,
            alt: 0,
            alt_gps: 0,
        };

        Self::magnitude(&p, &intersection)
    }

    pub fn douglas_peucker(pointlist: &Vec<FlightPoint>,epsilon: &f32) -> Vec<FlightPoint>
    {
        let mut dmax: f32 = 0.0;
        let mut index: u32 = 0;
        let mut cpt: u32 = 1;
        let end: usize = pointlist.len();

        if end < 3
        {
            return pointlist.to_vec();
        }

        for pt in pointlist.get(1..end).unwrap()
        {
            // let d1: f64 = Location::new(pt.lat, pt.long)
            //     .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            // let d2: f64 = Location::new(pt.lat, pt.long)
            //     .distance_to(&Location::new(pointlist[end - 1].lat, pointlist[end - 1].long)).unwrap().meters();

            let d = Self::distance_point_line(pt,&pointlist[0],pointlist.last().unwrap());

            if d > dmax
            {
                dmax = d;
                index = cpt;
            }
            cpt += 1;
        }

        let mut result = Vec::new();

        if dmax > *epsilon
        {
            let res1 = Self::douglas_peucker(&pointlist.get(..index as usize).unwrap().to_vec(), epsilon);
            let mut res2 = Self::douglas_peucker(&pointlist.get(index as usize..).unwrap().to_vec(), epsilon);
        
            result = res1;
            result.append(&mut res2);
        }else {
            result.push(pointlist[0].clone());
            result.push(pointlist[end - 1].clone());
        }

        return result;
    }

    
}

// DouglasPeucker pseudo code
// function DouglasPeucker(PointList[], epsilon)
//     # Find the point with the maximum distance
//     dmax = 0
//     index = 0
//     end = length(PointList)
//     for i = 2 to (end - 1) {
//         d = perpendicularDistance(PointList[i], Line(PointList[1], PointList[end])) 
//         if (d > dmax) {
//             index = i
//             dmax = d
//         }
//     }

//     ResultList[] = empty;

//     # If max distance is greater than epsilon, recursively simplify
//     if (dmax > epsilon) {
//         # Recursive call
//         recResults1[] = DouglasPeucker(PointList[1...index], epsilon)
//         recResults2[] = DouglasPeucker(PointList[index...end], epsilon)

//         # Build the result list
//         ResultList[] = {recResults1[1...length(recResults1) - 1], recResults2[1...length(recResults2)]}
//     } else {
//         ResultList[] = {PointList[1], PointList[end]}
//     }
//     # Return the result
//     return ResultList[]



// float Magnitude( XYZ *Point1, XYZ *Point2 )
// {
//     XYZ Vector;

//     Vector.X = Point2->X - Point1->X;
//     Vector.Y = Point2->Y - Point1->Y;
//     Vector.Z = Point2->Z - Point1->Z;

//     return (float)sqrt( Vector.X * Vector.X + Vector.Y * Vector.Y + Vector.Z * Vector.Z );
// }

// int DistancePointLine( XYZ *Point, XYZ *LineStart, XYZ *LineEnd, float *Distance )
// {
//     float LineMag;
//     float U;
//     XYZ Intersection;
 
//     LineMag = Magnitude( LineEnd, LineStart );
 
//     U = ( ( ( Point->X - LineStart->X ) * ( LineEnd->X - LineStart->X ) ) +
//         ( ( Point->Y - LineStart->Y ) * ( LineEnd->Y - LineStart->Y ) ) +
//         ( ( Point->Z - LineStart->Z ) * ( LineEnd->Z - LineStart->Z ) ) ) /
//         ( LineMag * LineMag );
 
//     if( U < 0.0f || U > 1.0f )
//         return 0;   // closest point does not fall within the line segment
 
//     Intersection.X = LineStart->X + U * ( LineEnd->X - LineStart->X );
//     Intersection.Y = LineStart->Y + U * ( LineEnd->Y - LineStart->Y );
//     Intersection.Z = LineStart->Z + U * ( LineEnd->Z - LineStart->Z );
 
//     *Distance = Magnitude( Point, &Intersection );
 
//     return 1;
// }