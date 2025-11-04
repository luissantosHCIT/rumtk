/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */
#![feature(inherent_associated_types)]
#![feature(rustc_private)]
#![feature(str_as_str)]
extern crate rumtk_core;
pub mod hl7_v2_base_types;
pub mod hl7_v2_complex_types;
pub mod hl7_v2_constants;
pub mod hl7_v2_field_descriptors;
pub mod hl7_v2_interpreter;
pub mod hl7_v2_mllp;
mod hl7_v2_optionality_rules;
pub mod hl7_v2_parser;
mod hl7_v2_scripting;
pub mod hl7_v2_search;
pub mod hl7_v2_types;
/*****************************************Tests****************************************/
#[cfg(test)]
mod tests {
    use crate::hl7_v2_base_types::v2_base_types::{
        V2DateTime, V2ParserCharacters, V2SearchIndex, V2String,
    };
    use crate::hl7_v2_base_types::v2_primitives::{
        V2PrimitiveCasting, V2PrimitiveType, TRUNCATE_FT,
    };
    use crate::hl7_v2_complex_types::hl7_v2_complex_types::{cast_component, V2Type};
    use crate::hl7_v2_constants::{V2_SEGMENT_IDS, V2_SEGMENT_NAMES};
    use crate::hl7_v2_field_descriptors::v2_field_descriptor::{
        V2ComponentType, V2ComponentTypeDescriptor,
    };
    use crate::hl7_v2_mllp::mllp_v2::{mllp_decode, mllp_encode, CR, EB, MLLP_FILTER_POLICY, SB};
    use crate::hl7_v2_optionality_rules::Optionality;
    use crate::hl7_v2_parser::v2_parser::{V2Field, V2Message};
    use crate::hl7_v2_search::REGEX_V2_SEARCH_DEFAULT;
    use crate::{
        rumtk_v2_find_component, rumtk_v2_generate_message, rumtk_v2_mllp_connect,
        rumtk_v2_mllp_get_client_ids, rumtk_v2_mllp_get_ip_port, rumtk_v2_mllp_iter_channels,
        rumtk_v2_mllp_listen, rumtk_v2_mllp_send, rumtk_v2_parse_message, tests,
    };
    use rumtk_core::core::RUMResult;
    use rumtk_core::search::rumtk_search::{string_search_named_captures, SearchGroups};
    use rumtk_core::strings::{
        basic_escape, format_compact, AsStr, RUMArrayConversions, RUMString, RUMStringConversions,
        StringUtils,
    };
    use rumtk_core::{
        rumtk_create_task, rumtk_deserialize, rumtk_exec_task, rumtk_init_threads, rumtk_serialize,
        rumtk_sleep,
    };
    use std::thread::spawn;
    /**********************************Constants**************************************/
    const DEFAULT_HL7_V2_MESSAGE: &str =
        "MSH|^~\\&|ADT1|GOOD HEALTH HOSPITAL|GHH LAB, INC.|GOOD HEALTH HOSPITAL|198808181126|SECURITY|ADT^A01^ADT_A01|MSG00001|P|2.8||\r\n\
         EVN|A01|200708181123||\n\
         PID|1||PATID1234^5^M11^ADT1^MR^GOOD HEALTH HOSPITAL~123456789^^^USSSA^SS||EVERYMAN^ADAM^A^III||19610615|M||C|2222 HOME STREET^^GREENSBORO^NC^27401-1020|GL|(555) 555-2004|(555)555-2004||S||PATID12345001^2^M10^ADT1^AN^A|444333333|987654^NC|\r\
         NK1|1|NUCLEAR^NELDA^W|SPO^SPOUSE||||NK^NEXT OF KIN\n\r\
         PV1|1|I|2000^2012^01||||004777^ATTEND^AARON^A|||SUR||||ADM|A0|";
    const VXU_HL7_V2_MESSAGE: &str =
        "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n
        PID|1||21142^^^NIST-MPI-1^MR||Vasquez^Manuel^Diego^^^^L||19470215|M||2106-3^White^CDCREC|227 Park Ave^^Bozeman^MT^59715^USA^P||^PRN^PH^^^406^5555815~^NET^^Manuel.Vasquez@isp.com|||||||||2135-2^Hispanic or Latino^CDCREC||N|1|||||N\n
        PD1|||||||||||01^No reminder/recall^HL70215|N|20150625|||A|20150625|20150625ORC|RE||31165^NIST-AA-IZ-2|||||||7824^Jackson^Lily^Suzanne^^^^^NIST-PI-1^L^^^PRN|||||||NISTEHRFAC^NISTEHRFacility^HL70362\n
        RXA|0|1|20141021||152^Pneumococcal Conjugate, unspecified formulation^CVX|999|||01^Historical Administration^NIP001|||||||||||CP|A";
    const HL7_V2_MESSAGE: &str =
        "FHS|^~\\&|WIR11.3.2|WIR|||20200514||1219274.update|||\n
        BHS|^~\\&|WIR11.3.2|WIR|||20200514|||||\n
        MSH|^~\\&|WIR11.3.2^^|WIR^^||WIRPH^^|20200514||VXU^V04^VXU_V04|2020051412382900|P^|2.5.1^^|||ER||||||^CDCPHINVS\n
        PID|||3064985^^^^SR^~ML288^^^^PI^||CHILD^BABEE^^^^^^||20180911|F||2106-3^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^WI027^^||(920)386-5555^PRN^PH^^^920^3865555^^|||||||||2186-5^^^^^|||||||\n
        PD1|||||||||||02^^^^^|U||||A\n
        NK1|1|CHILD^MARY^^^^^^|PAR^PARENT^HL70063^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^^^|(920)386-5555^PRN^PH^^^920^3865555^^\n
        NK1|2|CHILD^BABEE^^^^^^|SEL^SELF^HL70063^^^^^|22 YOUNGER LAND^^JUNEAU^WI^53039^^^^^^|(920)386-5555^PRN^PH^^^920^3865555^^\n
        PV1||R||||||||||||||||||\n
        ORC|RE||0|\n
        RXA|0|999|20190503|20190503|110^DTAP/Polio/Hep B^CVX^Pediarix^Pediarix^WVTN|1.0|||01^^^^^~38230637^WIR immunization id^IMM_ID^^^||^^^WIR Physicians^^^^^^^^^^^|||||||||\n
        BTS|1|\n
        FTS|1|";
    const HL7_V2_PDF_MESSAGE: &str =
        "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|201607060811||ORU^R03|5209141|D|2.3\n
        PID|1|123456|123456||Andrés^Ángel^||19600101|M|||123 BLACK PEARL^^DETROIT^MI^48217||3138363978|||||1036557|123456789\n
        PV1|||^^\n
        ORC|RE||161810162||||00001^ONCE^^^^||201607060811|||00007553^PHYSICIAN^आरवा^ ^^^|||||||||BIOTECH CLINICAL LAB INC^^23D0709666^^^CLIA|25775 MEADOWBROOK^^NOVI^MI^48375|^^^^^248^9121700|OFFICE^1234 MILE ROAD  SUITE # 2^ROYAL OAK^MI^48073\n
        OBR|8|455648^LA01|1618101622^GROUP951|GROUP951^CHROMOSOMAL ANALYSIS^L|||201606291253|||||||201606291631||00007553^PHYSICIAN^ひなた^ ^^^||||||201607060811|||F||1^^^^^9\n
        OBX|1|ED|00008510^INTELLIGENT FLOW PROFILE^L||^^^^JVBERi0xLjQKJeLjz9MKCjEgMCBvYmoKPDwvVHlwZSAvQ2F0YWxvZwovUGFnZXMgMiAwIFI+PgplbmRvYmoKCjIgMCBvYmoKPDwvVHlwZSAvUGFnZXMKL0tpZHMgWzMgMCBSXQovQ291bnQgMT4+CmVuZG9iagoKMTIgMCBvYmoKPDwvTGVuZ3RoIDEzIDAgUgovRmlsdGVyIC9GbGF0ZURlY29kZT4+CnN0cmVhbQp4nM1c3ZPUOJJ/569QxMRFMLtgrG+buBfoHmZ6BxgW+u5iN3hxV7m7feMqF3Y...T0YK||||||C|||201606291631\n
        NTE|1|L|Reference Lab: GENOPTIX|L\n
        NTE|2|L|2110 ROUTHERFORD RD|L\n
        NTE|3|L|CARLSBAD, CA  92008|L";
    // TODO: PID1.3 => See if it is the hospital mrn then concat with time (down to millis) at network receipt of message and concat with message id (incrementable) make arc<mutex<uint128>> or atomic integer.
    const HL7_V2_REPEATING_FIELD_MESSAGE: &str =
        "MSH|^~\\&#|NIST EHR^2.16.840.1.113883.3.72.5.22^ISO|NIST EHR Facility^2.16.840.1.113883.3.72.5.23^ISO|NIST Test Lab APP^2.16.840.1.113883.3.72.5.20^ISO|NIST Lab Facility^2.16.840.1.113883.3.72.5.21^ISO|20130211184101-0500||OML^O21^OML_O21|NIST-LOI_9.0_1.1-GU_PRU|T|2.5.1|||AL|AL|||||LOI_Common_Component^LOI BaseProfile^2.16.840.1.113883.9.66^ISO~LOI_GU_Component^LOI GU Profile^2.16.840.1.113883.9.78^ISO~LAB_PRU_Component^LOI PRU Profile^2.16.840.1.113883.9.82^ISO\n
        PID|1||PATID14567^^^NIST MPI&2.16.840.1.113883.3.72.5.30.2&ISO^MR||Hernandez^Maria^^^^^L||19880906|F||2054-5^Black or   African American^HL70005|3248 E  FlorenceAve^^Huntington Park^CA^90255^^H||^^PH^^^323^5825421|||||||||H^Hispanic or Latino^HL70189\n
        ORC|NW|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130116090021-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        OBR|1|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||34555-3^Creatinine 24H renal clearance panel^LN^^^^^^CreatinineClearance|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2\n
        DG1|2||O10.93^Unspecified pre-existing hypertension complicating the puerperium^I10C^^^^^^Pregnancy with chronic hypertension|||W|||||||||1\n
        OBX|1|CWE|67471-3^Pregnancy status^LN^1903^Pregnancy status^99USL^2.44^^Isthe patient pregnant?||Y^Yes^HL70136^1^Yes, confirmed less than 12 weeks^99USL^2.5.1^^early pregnancy (pre 12 weeks)||||||O|||20130115|||||||||||||||SCI\n
        OBX|2|NM|3167-4^Volume of   24   hour Urine^LN^1904^Urine Volume of 24 hour collection^99USL^2.44^^Urine Volume 24hour collection||1250|mL^milliliter^UCUM^ml^mililiter^L^1.7^^ml|||||O|||20130116|||||||||||||||SCI\n
        OBX|3|NM|3141-9^Body weight Measured^LN^BWm^Body weight Measured^99USL^2.44^^patient weight measured in kg||59.5|kg^kilogram^UCUM|||||O|||20130116|||||||||||||||SCI\n
        SPM|1|S-2312987-1&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||276833005^24 hour urine sample (specimen)^SCT^UR24H^24hr Urine^99USL^^^24 hour urine|||||||||||||201301151130-0800^201301160912-0800\n
        SPM|2|S-2312987-2&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||119297000^Blood Specimen^SCT|||||||||||||201301160912-0800ORC|NW|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130115102146-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        OBR|2|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||21482-5^Protein [Mass/volume] in 24 hour Urine^LN^^^^^^24 hour Urine Protein|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\n
        DG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2";
    const HL7_V2_MSH_ONLY: &str =
        "MSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC\n";
    const HL7_V2_SCRAMBLED: &str = "
                PD1|||||||||||01^No reminder/recall^HL70215|N|20150625|||A|20150625|20150625ORC|RE||31165^NIST-AA-IZ-2|||||||7824^Jackson^Lily^Suzanne^^^^^NIST-PI-1^L^^^PRN|||||||NISTEHRFAC^NISTEHRFacility^HL70362\rRXA|0|1|20141021||152^Pneumococcal Conjugate, unspecified formulation^CVX|999|||01^Historical Administration^NIP001|||||||||||CP|A\rPID|1||21142^^^NIST-MPI-1^MR||Vasquez^Manuel^Diego^^^^L||19470215|M||2106-3^White^CDCREC|227 Park Ave^^Bozeman^MT^59715^USA^P||^PRN^PH^^^406^5555815~^NET^^Manuel.Vasquez@isp.com|||||||||2135-2^Hispanic or Latino^CDCREC||N|1|||||N\rMSH|^~\\&|NISTEHRAPP|NISTEHRFAC|NISTIISAPP|NISTIISFAC|20150625072816.601-0500||VXU^V04^VXU_V04|NIST-IZ-AD-10.1_Send_V04_Z22|P|2.5.1|||ER|AL|||||Z22^CDCPHINVS|NISTEHRFAC|NISTIISFAC";
    const ESCAPED_V2_JSON_MESSAGE: &str = "{\"separators\":{\"segment_terminator\":\"\\r\",\"field_separator\":\"|\",\"component_separator\":\"^\",\"repetition_separator\":\"~\",\"escape_character\":\"\\\",\"subcomponent_separator\":\"&\",\"truncation_character\":\"#\"},\"segment_groups\":{\"113\":[{\"name\":\"ORC\",\"description\":\"Common Order\",\"fields\":[[{\"components\":[{\"component\":\"NW\"}]}],[{\"components\":[{\"component\":\"ORD231-1\"},{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.24\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"20130116090021-0800\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"134569827\"},{\"component\":\"Feller\"},{\"component\":\"Hans\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI&2.16.840.1.113883.4.6&ISO\"},{\"component\":\"L\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI\"}]}]]}],\"124\":[{\"name\":\"PID\",\"description\":\"Patient Identification\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"PATID14567\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NIST MPI&2.16.840.1.113883.3.72.5.30.2&ISO\"},{\"component\":\"MR\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"Hernandez\"},{\"component\":\"Maria\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"L\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"19880906\"}]}],[{\"components\":[{\"component\":\"F\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"2054-5\"},{\"component\":\"Black or   African American\"},{\"component\":\"HL70005\"}]}],[{\"components\":[{\"component\":\"3248 E  FlorenceAve\"},{\"component\":\"\"},{\"component\":\"Huntington Park\"},{\"component\":\"CA\"},{\"component\":\"90255\"},{\"component\":\"\"},{\"component\":\"H\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"PH\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"323\"},{\"component\":\"5825421\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"H\"},{\"component\":\"Hispanic or Latino\"},{\"component\":\"HL70189\"}]}]]}],\"38\":[{\"name\":\"DG1\",\"description\":\"Diagnosis\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"I10\"},{\"component\":\"Essential (primary) hypertension\"},{\"component\":\"I10C\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"Hypertension, NOS\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"F\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"2\"}]}]]},{\"name\":\"DG1\",\"description\":\"Diagnosis\",\"fields\":[[{\"components\":[{\"component\":\"2\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"O10.93\"},{\"component\":\"Unspecified pre-existing hypertension complicating the puerperium\"},{\"component\":\"I10C\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"Pregnancy with chronic hypertension\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"W\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"1\"}]}]]},{\"name\":\"DG1\",\"description\":\"Diagnosis\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"I10\"},{\"component\":\"Essential (primary) hypertension\"},{\"component\":\"I10C\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"Hypertension, NOS\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"F\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"2\"}]}]]}],\"97\":[{\"name\":\"OBR\",\"description\":\"Observation Request\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"ORD231-1\"},{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.24\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"34555-3\"},{\"component\":\"Creatinine 24H renal clearance panel\"},{\"component\":\"LN\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"CreatinineClearance\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"201301151130-0800\"}]}],[{\"components\":[{\"component\":\"201301160912-0800\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"134569827\"},{\"component\":\"Feller\"},{\"component\":\"Hans\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI&2.16.840.1.113883.4.6&ISO\"},{\"component\":\"L\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI\"}]}]]},{\"name\":\"OBR\",\"description\":\"Observation Request\",\"fields\":[[{\"components\":[{\"component\":\"2\"}]}],[{\"components\":[{\"component\":\"ORD231-2\"},{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.24\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"21482-5\"},{\"component\":\"Protein [Mass/volume] in 24 hour Urine\"},{\"component\":\"LN\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"24 hour Urine Protein\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"201301151130-0800\"}]}],[{\"components\":[{\"component\":\"201301160912-0800\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"134569827\"},{\"component\":\"Feller\"},{\"component\":\"Hans\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI&2.16.840.1.113883.4.6&ISO\"},{\"component\":\"L\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI\"}]}]]}],\"98\":[{\"name\":\"OBX\",\"description\":\"Observation/Result\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"CWE\"}]}],[{\"components\":[{\"component\":\"67471-3\"},{\"component\":\"Pregnancy status\"},{\"component\":\"LN\"},{\"component\":\"1903\"},{\"component\":\"Pregnancy status\"},{\"component\":\"99USL\"},{\"component\":\"2.44\"},{\"component\":\"\"},{\"component\":\"Isthe patient pregnant?\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"Y\"},{\"component\":\"Yes\"},{\"component\":\"HL70136\"},{\"component\":\"1\"},{\"component\":\"Yes, confirmed less than 12 weeks\"},{\"component\":\"99USL\"},{\"component\":\"2.5.1\"},{\"component\":\"\"},{\"component\":\"early pregnancy (pre 12 weeks)\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"O\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"20130115\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"SCI\"}]}]]},{\"name\":\"OBX\",\"description\":\"Observation/Result\",\"fields\":[[{\"components\":[{\"component\":\"2\"}]}],[{\"components\":[{\"component\":\"NM\"}]}],[{\"components\":[{\"component\":\"3167-4\"},{\"component\":\"Volume of   24   hour Urine\"},{\"component\":\"LN\"},{\"component\":\"1904\"},{\"component\":\"Urine Volume of 24 hour collection\"},{\"component\":\"99USL\"},{\"component\":\"2.44\"},{\"component\":\"\"},{\"component\":\"Urine Volume 24hour collection\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"1250\"}]}],[{\"components\":[{\"component\":\"mL\"},{\"component\":\"milliliter\"},{\"component\":\"UCUM\"},{\"component\":\"ml\"},{\"component\":\"mililiter\"},{\"component\":\"L\"},{\"component\":\"1.7\"},{\"component\":\"\"},{\"component\":\"ml\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"O\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"20130116\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"SCI\"}]}]]},{\"name\":\"OBX\",\"description\":\"Observation/Result\",\"fields\":[[{\"components\":[{\"component\":\"3\"}]}],[{\"components\":[{\"component\":\"NM\"}]}],[{\"components\":[{\"component\":\"3141-9\"},{\"component\":\"Body weight Measured\"},{\"component\":\"LN\"},{\"component\":\"BWm\"},{\"component\":\"Body weight Measured\"},{\"component\":\"99USL\"},{\"component\":\"2.44\"},{\"component\":\"\"},{\"component\":\"patient weight measured in kg\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"59.5\"}]}],[{\"components\":[{\"component\":\"kg\"},{\"component\":\"kilogram\"},{\"component\":\"UCUM\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"O\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"20130116\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"SCI\"}]}]]}],\"178\":[{\"name\":\"SPM\",\"description\":\"Specimen\",\"fields\":[[{\"components\":[{\"component\":\"1\"}]}],[{\"components\":[{\"component\":\"S-2312987-1&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"276833005\"},{\"component\":\"24 hour urine sample (specimen)\"},{\"component\":\"SCT\"},{\"component\":\"UR24H\"},{\"component\":\"24hr Urine\"},{\"component\":\"99USL\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"24 hour urine\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"201301151130-0800\"},{\"component\":\"201301160912-0800\"}]}]]},{\"name\":\"SPM\",\"description\":\"Specimen\",\"fields\":[[{\"components\":[{\"component\":\"2\"}]}],[{\"components\":[{\"component\":\"S-2312987-2&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"119297000\"},{\"component\":\"Blood Specimen\"},{\"component\":\"SCT\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"201301160912-0800ORC\"}]}],[{\"components\":[{\"component\":\"NW\"}]}],[{\"components\":[{\"component\":\"ORD231-2\"},{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.24\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"20130115102146-0800\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"134569827\"},{\"component\":\"Feller\"},{\"component\":\"Hans\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI&2.16.840.1.113883.4.6&ISO\"},{\"component\":\"L\"},{\"component\":\"\"},{\"component\":\"\"},{\"component\":\"NPI\"}]}]]}],\"89\":[{\"name\":\"MSH\",\"description\":\"Message Header\",\"fields\":[[{\"components\":[{\"component\":\"^~\\&#\"}]}],[{\"components\":[{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.22\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST EHR Facility\"},{\"component\":\"2.16.840.1.113883.3.72.5.23\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST Test Lab APP\"},{\"component\":\"2.16.840.1.113883.3.72.5.20\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST Lab Facility\"},{\"component\":\"2.16.840.1.113883.3.72.5.21\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"20130211184101-0500\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"OML\"},{\"component\":\"O21\"},{\"component\":\"OML_O21\"}]}],[{\"components\":[{\"component\":\"NIST-LOI_9.0_1.1-GU_PRU\"}]}],[{\"components\":[{\"component\":\"T\"}]}],[{\"components\":[{\"component\":\"2.5.1\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"AL\"}]}],[{\"components\":[{\"component\":\"AL\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"LOI_Common_Component\"},{\"component\":\"LOI BaseProfile\"},{\"component\":\"2.16.840.1.113883.9.66\"},{\"component\":\"ISO\"}]},{\"components\":[{\"component\":\"LOI_GU_Component\"},{\"component\":\"LOI GU Profile\"},{\"component\":\"2.16.840.1.113883.9.78\"},{\"component\":\"ISO\"}]},{\"components\":[{\"component\":\"LAB_PRU_Component\"},{\"component\":\"LOI PRU Profile\"},{\"component\":\"2.16.840.1.113883.9.82\"},{\"component\":\"ISO\"}]}]]}]}}";
    const V2_JSON_MESSAGE: &str = "MSH|^~\\&#|NIST EHR^2.16.840.1.113883.3.72.5.22^ISO|NIST EHR Facility^2.16.840.1.113883.3.72.5.23^ISO|NIST Test Lab APP^2.16.840.1.113883.3.72.5.20^ISO|NIST Lab Facility^2.16.840.1.113883.3.72.5.21^ISO|20130211184101-0500||OML^O21^OML_O21|NIST-LOI_9.0_1.1-GU_PRU|T|2.5.1|||AL|AL|||||LOI_Common_Component^LOI BaseProfile^2.16.840.1.113883.9.66^ISO~LOI_GU_Component^LOI GU Profile^2.16.840.1.113883.9.78^ISO~LAB_PRU_Component^LOI PRU Profile^2.16.840.1.113883.9.82^ISO\nPID|1||PATID14567^^^NIST MPI&2.16.840.1.113883.3.72.5.30.2&ISO^MR||Hernandez^Maria^^^^^L||19880906|F||2054-5^Black or   African American^HL70005|3248 E  FlorenceAve^^Huntington Park^CA^90255^^H||^^PH^^^323^5825421|||||||||H^Hispanic or Latino^HL70189\nORC|NW|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130116090021-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\nOBR|1|ORD231-1^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||34555-3^Creatinine 24H renal clearance panel^LN^^^^^^CreatinineClearance|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\nDG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2\nDG1|2||O10.93^Unspecified pre-existing hypertension complicating the puerperium^I10C^^^^^^Pregnancy with chronic hypertension|||W|||||||||1\nOBX|1|CWE|67471-3^Pregnancy status^LN^1903^Pregnancy status^99USL^2.44^^Isthe patient pregnant?||Y^Yes^HL70136^1^Yes, confirmed less than 12 weeks^99USL^2.5.1^^early pregnancy (pre 12 weeks)||||||O|||20130115|||||||||||||||SCI\nOBX|2|NM|3167-4^Volume of   24   hour Urine^LN^1904^Urine Volume of 24 hour collection^99USL^2.44^^Urine Volume 24hour collection||1250|mL^milliliter^UCUM^ml^mililiter^L^1.7^^ml|||||O|||20130116|||||||||||||||SCI\nOBX|3|NM|3141-9^Body weight Measured^LN^BWm^Body weight Measured^99USL^2.44^^patient weight measured in kg||59.5|kg^kilogram^UCUM|||||O|||20130116|||||||||||||||SCI\nSPM|1|S-2312987-1&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||276833005^24 hour urine sample (specimen)^SCT^UR24H^24hr Urine^99USL^^^24 hour urine|||||||||||||201301151130-0800^201301160912-0800\nSPM|2|S-2312987-2&NIST EHR&2.16.840.1.113883.3.72.5.24&ISO||119297000^Blood Specimen^SCT|||||||||||||201301160912-0800ORC|NW|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO|||||||20130115102146-0800|||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\nOBR|2|ORD231-2^NIST EHR^2.16.840.1.113883.3.72.5.24^ISO||21482-5^Protein [Mass/volume] in 24 hour Urine^LN^^^^^^24 hour Urine Protein|||201301151130-0800|201301160912-0800||||||||134569827^Feller^Hans^^^^^^NPI&2.16.840.1.113883.4.6&ISO^L^^^NPI\nDG1|1||I10^Essential (primary) hypertension^I10C^^^^^^Hypertension, NOS|||F|||||||||2";
    const ESCAPED_V2_JSON_MESSAGE_BASIC: &str = "{\"separators\":{\"segment_terminator\":\"\\r\",\"field_separator\":\"|\",\"component_separator\":\"^\",\"repetition_separator\":\"~\",\"escape_character\":\"\\\\\",\"subcomponent_separator\":\"&\",\"truncation_character\":\"#\"},\"segment_groups\":{\"89\":[{\"name\":\"MSH\",\"description\":\"Message Header\",\"fields\":[[{\"components\":[{\"component\":\"^~\\\\&#\"}]}],[{\"components\":[{\"component\":\"NIST EHR\"},{\"component\":\"2.16.840.1.113883.3.72.5.22\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST EHR Facility\"},{\"component\":\"2.16.840.1.113883.3.72.5.23\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST Test Lab APP\"},{\"component\":\"2.16.840.1.113883.3.72.5.20\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"NIST Lab Facility\"},{\"component\":\"2.16.840.1.113883.3.72.5.21\"},{\"component\":\"ISO\"}]}],[{\"components\":[{\"component\":\"20130211184101-0500\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"OML\"},{\"component\":\"O21\"},{\"component\":\"OML_O21\"}]}],[{\"components\":[{\"component\":\"NIST-LOI_9.0_1.1-GU_PRU\"}]}],[{\"components\":[{\"component\":\"T\"}]}],[{\"components\":[{\"component\":\"2.5.1\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"AL\"}]}],[{\"components\":[{\"component\":\"AL\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"\"}]}],[{\"components\":[{\"component\":\"LOI_Common_Component\"},{\"component\":\"LOI BaseProfile\"},{\"component\":\"2.16.840.1.113883.9.66\"},{\"component\":\"ISO\"}]},{\"components\":[{\"component\":\"LOI_GU_Component\"},{\"component\":\"LOI GU Profile\"},{\"component\":\"2.16.840.1.113883.9.78\"},{\"component\":\"ISO\"}]},{\"components\":[{\"component\":\"LAB_PRU_Component\"},{\"component\":\"LOI PRU Profile\"},{\"component\":\"2.16.840.1.113883.9.82\"},{\"component\":\"ISO\"}]}]]}]}}";
    const V2_JSON_MESSAGE_BASIC: &str = "MSH|^~\\&#|NIST EHR^2.16.840.1.113883.3.72.5.22^ISO|NIST EHR Facility^2.16.840.1.113883.3.72.5.23^ISO|NIST Test Lab APP^2.16.840.1.113883.3.72.5.20^ISO|NIST Lab Facility^2.16.840.1.113883.3.72.5.21^ISO|20130211184101-0500||OML^O21^OML_O21|NIST-LOI_9.0_1.1-GU_PRU|T|2.5.1|||AL|AL|||||LOI_Common_Component^LOI BaseProfile^2.16.840.1.113883.9.66^ISO~LOI_GU_Component^LOI GU Profile^2.16.840.1.113883.9.78^ISO~LAB_PRU_Component^LOI PRU Profile^2.16.840.1.113883.9.82^ISO\n";
    const SPANISH_NAME: &str = "Andrés";
    const SANSKRIT_NAME: &str = "आरवा";
    const HIRAGANA_NAME: &str = "ひなた";
    const repeate_field1: &str = "ISO";
    const repeate_field2: &str = "LOI_GU_Component";
    const repeate_field3: &str = "LAB_PRU_Component";
    const DEFAULT_HL7_V2_FIELD_STRING: &str = "2000^2012^01";

    /*********************************Test Cases**************************************/
    #[test]
    fn test_hl7_v2_field_parsing() {
        let field_str = tests::DEFAULT_HL7_V2_FIELD_STRING;
        let encode_chars = V2ParserCharacters::new();
        let field = V2Field::from_str(&field_str, &encode_chars);
        println!("{:#?}", &field);
        assert_eq!(field.len(), 3, "Wrong number of components in field");
        println!(
            "Value in component {} => {}!",
            0,
            field.get(1).unwrap().as_str()
        );
        assert_eq!(
            field.get(1).unwrap().as_str(),
            "2000",
            "Wrong value in component!"
        );
        println!(
            "Value in component {} => {}!",
            1,
            field.get(2).unwrap().as_str()
        );
        assert_eq!(
            field.get(2).unwrap().as_str(),
            "2012",
            "Wrong value in component!"
        );
        println!(
            "Value in component {} => {}!",
            2,
            field.get(3).unwrap().as_str()
        );
        assert_eq!(
            field.get(3).unwrap().as_str(),
            "01",
            "Wrong value in component!"
        );
    }

    #[test]
    fn test_sanitize_hl7_v2_message() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        println!("{}", message);
        println!("{}", sanitized_message);
        assert!(
            message.contains('\n'),
            "Raw message has new line characters."
        );
        assert!(
            !sanitized_message.contains('\n'),
            "Sanitized message has new line characters."
        );
        assert!(!sanitized_message.contains("\r\r"), "Sanitizer failed to consolidate double carriage returns into a single carriage return per instance..");
    }

    #[test]
    fn test_tokenize_hl7_v2_message() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        println!("Token count {}", tokens.len());
        assert_eq!(
            tokens.len(),
            5,
            "Tokenizer generated the wrong number of tokens! We expected 5 segment tokens."
        );
    }

    #[test]
    fn test_load_hl7_v2_encoding_characters() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let encode_chars = V2ParserCharacters::from_msh(tokens[0]).unwrap();
        println!("{:#?}", encode_chars);
        assert!(
            encode_chars.segment_terminator.contains('\r'),
            "Wrong segment character!"
        );
        assert!(
            encode_chars.field_separator.contains('|'),
            "Wrong field character!"
        );
        assert!(
            encode_chars.component_separator.contains('^'),
            "Wrong component character!"
        );
        assert!(
            encode_chars.repetition_separator.contains('~'),
            "Wrong repetition character!"
        );
        assert!(
            encode_chars.escape_character.contains('\\'),
            "Wrong escape character!"
        );
        assert!(
            encode_chars.subcomponent_separator.contains('&'),
            "Wrong subcomponent character!"
        );
        assert!(
            encode_chars.truncation_character.contains('#'),
            "Wrong truncation character!"
        );
    }

    #[test]
    fn test_extract_hl7_v2_message_segments() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let msh = V2Message::find_msh(&tokens).unwrap();
        let encode_chars = V2ParserCharacters::from_msh(&msh.as_str()).unwrap();
        let parsed_segments = V2Message::extract_segments(&tokens, &encode_chars).unwrap();
        let keys = parsed_segments.keys();
        print!("Keys: ");
        for k in keys {
            print!("{} ", V2_SEGMENT_NAMES[k]);
        }
        assert_eq!(
            parsed_segments.len(),
            5,
            "Number of segments mismatching what was expected!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["PID"]),
            "Missing PID segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["PV1"]),
            "Missing PV1 segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["EVN"]),
            "Missing EVN segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["NK1"]),
            "Missing NK1 segment!"
        );
    }

    #[test]
    fn test_extract_hl7_v2_message_scrambled_segments() {
        let message = tests::HL7_V2_SCRAMBLED;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let msh = V2Message::find_msh(&tokens).unwrap();
        let encode_chars = V2ParserCharacters::from_msh(&msh.as_str()).unwrap();
        let parsed_segments = V2Message::extract_segments(&tokens, &encode_chars).unwrap();
        let keys = parsed_segments.keys();
        print!("Keys: ");
        for k in keys {
            print!("{} ", V2_SEGMENT_NAMES[k]);
        }
        assert_eq!(
            parsed_segments.len(),
            4,
            "Number of segments mismatching what was expected!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["PID"]),
            "Missing PID segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["PD1"]),
            "Missing PV1 segment!"
        );
        assert!(
            parsed_segments.contains_key(&V2_SEGMENT_IDS["RXA"]),
            "Missing EVN segment!"
        );
    }

    #[test]
    fn test_load_hl7_v2_message() {
        let message = V2Message::from_str(tests::DEFAULT_HL7_V2_MESSAGE);
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PID"]),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PV1"]),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["EVN"]),
            "Missing EVN segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["NK1"]),
            "Missing NK1 segment!"
        );
    }

    ///
    /// Per examples in https://confluence.hl7.org/display/OO/v2+Sample+Messages you can have
    ///  messages that have other header segments before the standard MSH header.
    ///  As a result, I have made the logic a bit more permissive of the position of the msh segment.
    ///  I also made sure segments were trimmed to avoid issues with white space padding
    ///
    #[test]
    fn test_load_hl7_v2_message_wir_iis() {
        let message = V2Message::from_str(tests::HL7_V2_MESSAGE);
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["FHS"]),
            "Missing FHS segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["NK1"]),
            "Missing NK1 segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PV1"]),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["FTS"]),
            "Missing FTS segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["BHS"]),
            "Missing BHS segment!"
        );
    }
    #[test]
    fn test_load_hl7_v2_message_scrambled() {
        let message = V2Message::from_str(tests::HL7_V2_SCRAMBLED);
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PID"]),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PD1"]),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["RXA"]),
            "Missing EVN segment!"
        );
    }

    ///
    /// Testing for the proper parsing of message when presented with Unicode portions.
    ///
    #[test]
    fn test_load_hl7_v2_utf8_message() {
        let message = V2Message::from_str(tests::HL7_V2_PDF_MESSAGE);
        let pid = message.get(&V2_SEGMENT_IDS["PID"], 1).unwrap();
        let orc = message.get(&V2_SEGMENT_IDS["ORC"], 1).unwrap();
        let obr = message.get(&V2_SEGMENT_IDS["OBR"], 1).unwrap();
        let name1 = pid.get(5).unwrap().get(0).unwrap().get(1).unwrap().as_str();
        let name2 = orc
            .get(12)
            .unwrap()
            .get(0)
            .unwrap()
            .get(3)
            .unwrap()
            .as_str();
        let name3 = obr
            .get(16)
            .unwrap()
            .get(0)
            .unwrap()
            .get(3)
            .unwrap()
            .as_str();
        println!("{}", name1);
        println!("{}", name2);
        println!("{}", name3);
        assert_eq!(name1, SPANISH_NAME, "Wrong name/string found in PID(1)5.1!");
        assert_eq!(
            name2, SANSKRIT_NAME,
            "Wrong name/string found in ORC(1)12.3!"
        );
        assert_eq!(
            name3, HIRAGANA_NAME,
            "Wrong name/string found in OBR(1)16.3!"
        );
    }

    ///
    /// Testing for the proper parsing of message when presented with repeating fields.
    ///
    #[test]
    fn test_handle_hl7_v2_message_with_repeating_fields() {
        let message = V2Message::from_str(tests::HL7_V2_REPEATING_FIELD_MESSAGE);
        let msh = message.get(&V2_SEGMENT_IDS["MSH"], 1).unwrap();
        let field1 = msh
            .get(-1)
            .unwrap()
            .get(0)
            .unwrap()
            .get(4)
            .unwrap()
            .as_str();
        let field2 = msh
            .get(-1)
            .unwrap()
            .get(1)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        let field3 = msh
            .get(-1)
            .unwrap()
            .get(2)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        assert_eq!(
            msh.get(-1).unwrap().len(),
            3,
            "Wrong number of subfields in group in MSH(1)-1!"
        );
        assert_eq!(
            field1, repeate_field1,
            "Wrong field contents found in MSH(1)-1(0).4!"
        );
        assert_eq!(
            field2, repeate_field2,
            "Wrong field contents found in MSH(1)-1(1).1!"
        );
        assert_eq!(
            field3, repeate_field3,
            "Wrong field contents found in MSH(1)-1(2).1!"
        );
    }

    #[test]
    fn test_generating_v2_message() {
        let message = rumtk_v2_parse_message!(&tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_wir() {
        let message = rumtk_v2_parse_message!(&tests::HL7_V2_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_pdf() {
        let message = rumtk_v2_parse_message!(&tests::HL7_V2_PDF_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_generating_v2_message_repeated_fields() {
        let message = rumtk_v2_parse_message!(&tests::HL7_V2_REPEATING_FIELD_MESSAGE).unwrap();
        let generated_message_string = rumtk_v2_generate_message!(&message);
        let generated_message = rumtk_v2_parse_message!(&generated_message_string).unwrap();
        assert_eq!(
            &message, &generated_message,
            "Messages are not equal! Expected: {:?} Got: {:?}",
            &message, &generated_message
        );
    }

    #[test]
    fn test_handle_hl7_v2_search_pattern_parsing_full() {
        let pattern = "MSH(1)-1[5].4";
        let groups = string_search_named_captures(pattern, REGEX_V2_SEARCH_DEFAULT, "1");
        let expected = SearchGroups::from([
            (RUMString::new("segment_group"), RUMString::new("1")),
            (RUMString::new("sub_field"), RUMString::new("5")),
            (RUMString::new("segment"), RUMString::new("MSH")),
            (RUMString::new("field"), RUMString::new("-1")),
            (RUMString::new("component"), RUMString::new("4")),
        ]);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            pattern, expected, groups
        );
        assert_eq!(
            groups, expected,
            "Misparsed search expression MSH(1)-1[5].4!"
        );
    }

    #[test]
    fn test_handle_hl7_v2_search_pattern_parsing_simple() {
        let pattern = "MSH1.4";
        let groups = string_search_named_captures(pattern, REGEX_V2_SEARCH_DEFAULT, "1");
        let expected = SearchGroups::from([
            (RUMString::new("segment_group"), RUMString::new("1")),
            (RUMString::new("sub_field"), RUMString::new("1")),
            (RUMString::new("segment"), RUMString::new("MSH")),
            (RUMString::new("field"), RUMString::new("1")),
            (RUMString::new("component"), RUMString::new("4")),
        ]);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            pattern, expected, groups
        );
        assert_eq!(groups, expected, "Misparsed search expression MSH1.4!");
    }

    #[test]
    fn test_v2_search_index() {
        let expr = "MSH(1)-1[5].4";
        let v2_search_index = V2SearchIndex::from(expr);
        let expected = V2SearchIndex::new("MSH", 1, -1, 5, 4);
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            expr, expected, v2_search_index
        );
        assert_eq!(
            v2_search_index, expected,
            "Failed to parse expression into correct SearchIndex object."
        );
    }

    #[test]
    fn test_load_hl7_v2_message_macro() {
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["MSH"]),
            "Missing MSH segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PID"]),
            "Missing PID segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["PV1"]),
            "Missing PV1 segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["EVN"]),
            "Missing EVN segment!"
        );
        assert!(
            message.segment_exists(&V2_SEGMENT_IDS["NK1"]),
            "Missing NK1 segment!"
        );
    }

    #[test]
    fn test_load_hl7_v2_message_macro_failure() {
        let input = "Hello World!";
        let err_msg = format_compact!(
            "Parsing did not fail as expected. Input {} => parsed?",
            input
        );
        match rumtk_v2_parse_message!(input) {
            Ok(v) => panic!("{}", err_msg.as_str()),
            Err(e) => {
                println!("{}", format_compact!("Got error => {}", e).as_str());
                println!("Passed failed case!");
            }
        };
    }

    #[test]
    fn test_find_hl7_v2_message_component_macro() {
        let pattern = "PID(1)5.4";
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "III";
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_component_simple_macro() {
        let pattern = "PID5.4";
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "III";
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_msh_field() {
        let pattern = "MSH1.1";
        let message = rumtk_v2_parse_message!(tests::HL7_V2_MSH_ONLY).unwrap();
        let component = rumtk_v2_find_component!(message, pattern).unwrap();
        let expected = "^~\\&";
        assert_eq!(
            component.as_str(),
            expected,
            "Wrong component found! Looked for {} expecting {}, but got {}",
            pattern,
            expected,
            component.as_str()
        );
    }

    #[test]
    fn test_find_hl7_v2_message_component_macro_failure() {
        let pattern = "PID(1)15.4";
        let err_msg = format_compact!(
            "Search did not fail as expected. Input {} => found component?",
            pattern
        );
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        match rumtk_v2_find_component!(message, pattern) {
            Ok(v) => panic!("{}", err_msg.as_str()),
            Err(e) => {
                println!("{}", format_compact!("Got error => {}", e).as_str());
                println!("Passed failed case!");
            }
        }
    }

    #[test]
    fn test_cast_component_to_datetime_expected_functionality() {
        let inputs = [
            "2007",
            "200708",
            "20070818",
            "200708181123",
            "20070818112355",
            "20070818112355.55",
            "20070818112355.5555-5000",
            "20070818112355-5000",
        ];
        let expected_outputs = [
            "2007-01-01T00:00:00.0000",
            "2007-08-01T00:00:00.0000",
            "2007-08-18T00:00:00.0000",
            "2007-08-18T11:23:00.0000",
            "2007-08-18T11:23:55.0000",
            "2007-08-18T11:23:55.5500",
            "2007-08-18T11:23:55.5555-5000",
            "2007-08-18T11:23:55.0000-5000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2datetime().unwrap();
            let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_datetime_validation() {
        let inputs = ["200"];
        for input in inputs {
            match input.to_v2datetime() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_datetime_base_example() {
        let location = "EVN2"; //EVN|A01|200708181123||\n\r; EVN2 => segment = EVN, field = 2
        let expected_component = "200708181123";
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, location).unwrap();
        assert_eq!(expected_component, component.as_str(), "We are not using the correct component for this test. Check that the original test message has not changed and update the location string appropriately!");
        let date = component.to_v2datetime().unwrap();
        let expected_utc = "2007-08-18T11:23:00.0000";
        let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [{}]", component.as_str());
        assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg)
    }

    #[test]
    fn test_datetime_default() {
        let input = V2DateTime::default().as_utc_string();
        let expected_val = V2String::from("1970-01-01T00:00:00.00000");
        let err_msg = format_compact!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, input);
        assert_eq!(expected_val, input, "{}", &err_msg);
    }

    #[test]
    fn test_cast_component_to_date_expected_functionality() {
        let inputs = ["2007", "200708", "20070818"];
        let expected_outputs = [
            "2007-01-01T00:00:00.0000",
            "2007-08-01T00:00:00.0000",
            "2007-08-18T00:00:00.0000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2date().unwrap();
            let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_date_validation() {
        let inputs = ["200"];
        for input in inputs {
            match input.to_v2date() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_date_base_example() {
        let location = "PD113"; //EVN|A01|200708181123||\n\r; PD113 => segment = PD1, field = 13
        let expected_component = "20150625";
        let message = rumtk_v2_parse_message!(tests::VXU_HL7_V2_MESSAGE).unwrap();
        let component = rumtk_v2_find_component!(message, location).unwrap();
        assert_eq!(expected_component, component.as_str(), "We are not using the correct component for this test. Check that the original test message has not changed and update the location string appropriately!");
        let date = component.to_v2date().unwrap();
        let expected_utc = "2015-06-25T00:00:00.0000";
        let err_msg = format_compact!(
            "The expected date string does not match the date string generated from the input [{}]",
            component.as_str()
        );
        assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg)
    }

    #[test]
    fn test_cast_component_to_time_expected_functionality() {
        let inputs = ["1123", "112355", "112355.5555", "112355.5555-5000"];
        let expected_outputs = [
            "1970-01-01T11:23:00.0000",
            "1970-01-01T11:23:55.0000",
            "1970-01-01T11:23:55.5555",
            "1970-01-01T11:23:55.5555-5000",
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_utc = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to datetime type.",
                i, input, expected_utc
            );
            let date = input.to_v2time().unwrap();
            let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, date.as_utc_string());
            assert_eq!(expected_utc, date.as_utc_string().as_str(), "{}", &err_msg);
            println!(" ... Got: {} ✅ ", date.as_utc_string());
        }
    }

    #[test]
    fn test_cast_component_to_time_validation() {
        let inputs = ["2"];
        for input in inputs {
            match input.to_v2time() {
                Ok(date) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input,
                        date.as_utc_string()
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_number_expected_functionality() {
        let inputs = [
            "5e3",
            "5E3",
            "112355.5555",
            "5F",
            "5.5F",
            "5f",
            "5.5e2",
            "-5f",
            "-05e1",
        ];
        let expected_outputs = [
            5000.0,
            5000.0,
            112355.5555,
            5.0,
            5.5,
            5.0,
            550.0,
            -5.0,
            -50.0,
        ];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to NM type.",
                i, input, expected_val
            );
            let val = input.to_v2number().unwrap();
            let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_cast_component_to_number_validation() {
        let inputs = [".2"];
        for input in inputs {
            match input.to_v2number() {
                Ok(val) => {
                    panic!(
                        "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                        input, val
                    );
                }
                Err(e) => println!(
                    "Validation correctly identified malformed input with message => [{}] ✅",
                    e.as_str()
                ),
            }
        }
    }

    #[test]
    fn test_cast_component_to_st_expected_functionality() {
        let inputs = [" Hello World!"];
        let expected_outputs = ["Hello World!"];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to ST type.",
                i, input, expected_val
            );
            let val = input.to_v2stringdata().unwrap();
            let err_msg = format_compact!("The expected date time string does not match the date time string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_cast_component_to_st_validation() {
        let input = "2".duplicate(1001);
        println!("{}", input);
        match input.to_v2stringdata() {
            Ok(val) => {
                panic!(
                    "Validation failed [In: {} Got: {} Expected: None] ... ✕",
                    input, val
                );
            }
            Err(e) => println!(
                "Validation correctly identified malformed input with message => [{}] ✅",
                e.as_str()
            ),
        }
    }

    #[test]
    fn test_cast_component_to_ft_expected_functionality() {
        let inputs = ["H", &"e".duplicate(120000)];
        let expected_outputs = ["H", &"e".duplicate(TRUNCATE_FT as usize)];
        for i in 0..inputs.len() {
            let input = inputs[i];
            let expected_val = expected_outputs[i];
            print!(
                "Testing input #{} \"{}\". Expected output is \"{}\". Casting to FT type.",
                i, input, expected_val
            );
            let val = input.to_v2formattedtext("~").unwrap();
            println!("{}", val.len());
            let err_msg = format_compact!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, val);
            assert_eq!(expected_val, val, "{}", &err_msg);
            println!(" ... Got: {} ✅ ", val);
        }
    }

    #[test]
    fn test_validated_cast_component_to_type() {
        let message = tests::DEFAULT_HL7_V2_MESSAGE;
        let sanitized_message = V2Message::sanitize(message);
        let tokens = V2Message::tokenize_segments(&sanitized_message.as_str());
        let encode_chars = V2ParserCharacters::from_msh(tokens[0]).unwrap();
        let v2_component = V2ComponentTypeDescriptor::new(
            "date",
            "Date",
            V2ComponentType::Primitive(V2PrimitiveType::Date),
            4,
            1,
            1,
            Optionality::O,
            true,
        );
        let input = "2007";
        let val = cast_component(vec![&input], &v2_component, &encode_chars);
        let expected = "2007-01-01T00:00:00.0000";
        let err_msg = format_compact!("The expected formatted string does not match the formatted string generated from the input [In: {}, Got: {}]", input, expected);

        match val {
            V2Type::V2Date(result) => {
                assert_eq!(expected, result.unwrap().as_utc_string(), "{}", &err_msg)
            }
            _ => panic!("Wrong type received!"),
        }
    }

    // TODO: Add tests for sequenceid and telephonestring
    // TODO: Add fuzzing test for to_datetime().

    #[test]
    fn test_mllp_encode() {
        let expected_message = RUMString::from("I ❤ my wife!");
        let encoded = mllp_encode(&expected_message);
        let payload = &encoded[1..encoded.len() - 2];

        assert_eq!(encoded[0], SB, "Incorrect start byte in MLLP message!");

        assert_eq!(
            encoded[encoded.len() - 2],
            EB,
            "Incorrect end byte in MLLP message!"
        );

        assert_eq!(
            encoded[encoded.len() - 1],
            CR,
            "Missing mandatory carriage return in MLLP message!"
        );

        assert_eq!(
            expected_message,
            payload.to_rumstring(),
            "{}",
            format_compact!(
                "Malformed payload! Expected: {} Found: {}",
                expected_message,
                payload.to_rumstring()
            )
        );
    }

    #[test]
    fn test_mllp_decode() {
        let expected_message = RUMString::from("I ❤ my wife!");
        let message_size = expected_message.len();
        let encoded = mllp_encode(&expected_message);
        let encoded_size = encoded.len();

        assert_eq!(
            encoded_size,
            message_size + 3,
            "Incorrect encoded message size!"
        );

        let decoded = mllp_decode(&encoded).unwrap();
        let decoded_size = decoded.len();

        assert_eq!(
            decoded_size, message_size,
            "Incorrect decoded message size! Expected: {} Got: {}",
            expected_message, decoded
        );

        assert_eq!(
            expected_message,
            decoded,
            "{}",
            format_compact!(
                "Malformed decoded message! Expected: {} Found: {}",
                expected_message,
                decoded
            )
        );
    }

    #[test]
    fn test_mllp_listen() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&mllp_layer);
        let client_id = rumtk_exec_task!(async || -> RUMResult<RUMString> {
            Ok(mllp_layer.lock().await.get_address_info().await.unwrap())
        });
        assert_eq!(
            client_id,
            Ok(format_compact!("127.0.0.1:{}", &port)),
            "Failed to bind local port!"
        )
    }

    #[test]
    fn test_mllp_get_ip() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&mllp_layer);
    }

    #[test]
    fn test_mllp_connect() {
        let mllp_layer = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&mllp_layer);
        let client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let mut connected_clients = rumtk_v2_mllp_get_client_ids!(&mllp_layer);
        for i in 0..10 {
            if connected_clients.is_empty() {
                rumtk_sleep!(1);
                connected_clients = rumtk_v2_mllp_get_client_ids!(&mllp_layer);
            }
        }
        let connected_address = connected_clients.get(0).unwrap();
        let client_ids = rumtk_v2_mllp_get_client_ids!(&client);
        let client_id = client_ids.get(0).unwrap();
        assert_eq!(connected_address, client_id, "Failed to bind local port!")
    }

    #[test]
    fn test_mllp_channel() {
        let empty_string = |s: RUMString| Ok::<RUMString, RUMString>(RUMString::from(""));
        let safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&safe_listener);
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(&safe_listener);
        let client_id = client_ids.get(0).unwrap();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(&safe_client);
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let channel_address = server_channel.lock().unwrap().get_address_info().unwrap();
        assert_eq!(
            &client_id,
            &channel_address,
            "{}",
            format_compact!(
                "Issue stablishing MLLP communication channel! Expected: {} Received: {}",
                &client_id,
                &channel_address
            )
        )
    }

    #[test]
    fn test_mllp_channel_async_communication() {
        let mut safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_layer) => mllp_layer,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&safe_listener);
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(&safe_listener);
        let client_id = client_ids.get(0).unwrap();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(&safe_client);
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let expected_message = RUMString::from("I ❤ my wife!");
        let message_copy = expected_message.clone();
        let send_thread = spawn(move || -> RUMResult<()> {
            Ok(server_channel
                .lock()
                .unwrap()
                .send_message(&message_copy)
                .unwrap())
        });
        rumtk_sleep!(1);
        let received_message = rumtk_exec_task!(async || -> RUMResult<RUMString> {
            let mut received_message = safe_listener
                .lock()
                .await
                .receive_message(&client_id)
                .await?;
            while received_message.len() == 0 {
                received_message = safe_listener
                    .lock()
                    .await
                    .receive_message(&client_id)
                    .await?;
            }
            Ok(received_message)
        })
        .unwrap();
        assert_eq!(
            &expected_message,
            &received_message,
            "{}",
            format_compact!(
                "Issue sending message through channel! Expected: {} Received: {}",
                &expected_message,
                &received_message
            )
        )
    }

    #[test]
    fn test_mllp_hl7_echo() {
        let empty_string = |s: RUMString| Ok::<RUMString, RUMString>(RUMString::from(""));
        let mut safe_listener = match rumtk_v2_mllp_listen!(0, MLLP_FILTER_POLICY::NONE, true) {
            Ok(mllp_listener) => mllp_listener,
            Err(e) => panic!("{}", e),
        };
        let (ip, port) = rumtk_v2_mllp_get_ip_port!(&safe_listener);
        let safe_client = match rumtk_v2_mllp_connect!(port, MLLP_FILTER_POLICY::NONE) {
            Ok(client) => client,
            Err(e) => panic!("{}", e),
        };
        rumtk_sleep!(1);
        let client_ids = rumtk_v2_mllp_get_client_ids!(&safe_listener);
        let client_id = client_ids.get(0).unwrap();
        let mut server_channels = rumtk_v2_mllp_iter_channels!(&safe_client);
        let mut server_channel = server_channels.get_mut(0).unwrap().clone();
        let server_channel_copy = server_channel.clone();
        let send_thread = spawn(move || -> RUMResult<()> {
            Ok(server_channel
                .lock()
                .unwrap()
                .send_message(HL7_V2_PDF_MESSAGE)
                .unwrap())
        });
        let safe_listener_copy = safe_listener.clone();
        let received_message = rumtk_exec_task!(async || -> RUMResult<RUMString> {
            let mut received_message = safe_listener_copy
                .lock()
                .await
                .receive_message(&client_id)
                .await?;
            while received_message.len() == 0 {
                received_message = safe_listener_copy
                    .lock()
                    .await
                    .receive_message(&client_id)
                    .await?;
            }
            Ok(received_message)
        })
        .unwrap();
        assert_eq!(
            &HL7_V2_PDF_MESSAGE,
            &received_message,
            "{}",
            format_compact!(
                "Issue sending message through channel! Expected: {} Received: {}",
                &HL7_V2_PDF_MESSAGE,
                &received_message
            )
        );
        let client_id_copy = client_id.clone();
        let safe_listener_copy2 = safe_listener.clone();
        println!("Echoing message back to client!");
        let echo_thread = spawn(move || {
            println!("Sending echo message!");
            rumtk_v2_mllp_send!(safe_listener_copy2, client_id_copy, HL7_V2_PDF_MESSAGE).unwrap();
            println!("Sent echo message!");
        });
        rumtk_sleep!(1);
        let echoed_message = rumtk_exec_task!(async || -> RUMResult<RUMString> {
            println!("Echoing message back to client!");
            let mut echoed_message = safe_client.lock().await.receive_message(&client_id).await?;
            while echoed_message.len() == 0 {
                echoed_message = safe_client.lock().await.receive_message(&client_id).await?;
            }
            println!("Echoed message: {}", &echoed_message);
            Ok(echoed_message)
        })
        .unwrap();
        assert_eq!(
            &HL7_V2_PDF_MESSAGE,
            &echoed_message,
            "{}",
            format_compact!(
                "Issue echoing message through channel! Expected: {} Received: {}",
                &HL7_V2_PDF_MESSAGE,
                &echoed_message
            )
        )
    }

    ////////////////////////////JSON Tests/////////////////////////////////
    #[test]
    fn test_deserialize_escaped_v2_message() {
        let message = rumtk_v2_parse_message!(tests::V2_JSON_MESSAGE).unwrap();
        let serialized = rumtk_serialize!(&message).unwrap();
        let escaped = basic_escape(&serialized);
        let deserialized = rumtk_deserialize!(&escaped).unwrap();

        assert_eq!(
            message, deserialized,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_deserialize_v2_message() {
        let message = rumtk_v2_parse_message!(tests::DEFAULT_HL7_V2_MESSAGE).unwrap();
        let message_str = rumtk_serialize!(&message, true).unwrap();
        let deserialized: V2Message = rumtk_deserialize!(&message_str).unwrap();

        assert_eq!(
            message, deserialized,
            "Deserialized JSON does not match the expected value!"
        );
    }

    #[test]
    fn test_deserialize_stdin_v2_message_basic() {
        let expected_message: V2Message =
            rumtk_v2_parse_message!(tests::V2_JSON_MESSAGE_BASIC).unwrap();
        let deserialized: V2Message =
            rumtk_deserialize!(&tests::ESCAPED_V2_JSON_MESSAGE_BASIC).unwrap();

        assert_eq!(
            expected_message, deserialized,
            "Deserialized Escaped JSON does not match the expected value!"
        );
    }

    ////////////////////////////Fuzzed Tests/////////////////////////////////

    #[test]
    fn test_fuzzed_garbage_parsing() {
        let input = "MSH@~��MS";
        match rumtk_v2_parse_message!(&input) {
            Err(e) => println!("Correctly identified input as garbage! => {}", &e),
            Ok(message) => {
                println!("Test input [{}] Result => {:?}", &input, message);
                panic!("Message parsed without errors despite being malformed!")
            }
        }
    }
}
