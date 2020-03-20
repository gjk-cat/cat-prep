//! modul obsahující jednotlivé objektu cat-prepu
//!
//! každý model spočívá v páru karta - profil,
//! kdy karta odpovídá TOMLu přečtenému z headeru
//! souboru (nebo ze souboru ucitel.toml)

use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Karta učitele
///
/// Tato struktura reprezentuje konfigurační soubor
/// `teachers/ucitel.toml`, který obsahuje základní
/// informace o vyučujícím.
///
/// Tento soubor je následně využíván k identifikace
/// autora, zodpovědných osob, vytvoření profilu
/// učitele a asociaci předmětů a jednotlivých materiálů
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherCard {
	/// jméno vyučujícího,
	/// mělo by být opravdové jméno
	/// a korespondovat s gitovou
	/// konfigurační hodnotou `user.name`
	pub jmeno:    String,
	/// email vyučujícího,
	/// měl by korespondovat
	/// s gitovou konfigurační hodnotou
	/// `user.email`
	pub email:    String,
	/// uživatelské jméno uživatele,
	/// nesmí obsahovat mezery,
	/// využit pro mezistránkové odkazy,
	/// další kritérium pro vyhledávání
	/// souborů v repozitáři modifikovaných/vytvořených
	/// uživatelem
	pub username: String,
	/// popisek vyučujícího.
	/// může obsahovat cokoliv,
	/// formátováno jako markdown
	pub bio:      String,
}

/// Karta článku
///
/// Tato struktura reprezentuje konfigurační
/// hodnoty daného článku ve formátu TOML,
/// odděleno řádkem `+++` od těla článku
///
/// využito při generování metadat článku,
/// a generování odkazů na mnoha jiných místech
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleCard {
	/// název článku,
	/// bude se objevovat v odkazech
	/// a nahoře v kartě článku
	pub nazev:          String,
	/// seznam tagů, které má daný článek
	/// využit pro vytvoření databáze tagů
	/// a následné nalinkování
	pub tagy:           Vec<String>,
	/// datum, může obsahovat cokoliv
	pub datum:          Option<String>,
	/// tato složka je pomocná a nemá být
	/// konfigurována v markdown souboru,
	/// jejím účelem je uchovávat cestu k souboru,
	/// ke kterému karta patří v kontextech,
	/// kde není použít [`Article`]
	/// (nebo dokud není sestavena databáze článků)
	pub _resolved_path: Option<PathBuf>,
}

/// Článek
///
/// Tato struktura reprezentuje všechna
/// metadata daného materiálu, slouží jako
/// hlavní zdroj informací.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
	/// karta článku
	pub card:              ArticleCard,
	/// kdy byl naposled článek modifikován;
	/// vypočítáno pomocí gitu.
	pub last_modified:     String,
	/// kým byl článek naposledy modifikován;
	/// informaca získané z gitu.
	pub modified_by:       String,
	/// autor daného materiálu,
	/// informace získaná z gitu,
	/// viz [`Teacher::files_created`].
	pub author:            String,
	/// cesta k článku,
	/// relativní ke složce `src`
	/// (a tudíž kořenovému adresáři webu)
	pub path:              PathBuf,
	/// pokud se podle [`Article::modified_by`] podařilo
	/// najít vyučujícího, bude zde uložena jeho karta
	pub modified_resolved: Option<TeacherCard>,
	/// pokud se podle [`Article::author`] podařilo
	/// najít vyučujícího, bude zde uložena jeho karta
	pub resolved_author:   Option<TeacherCard>,
	/// zde se nalézá přiřazená karta předmětu,
	/// typ `Option` je použit proto, protože v době
	/// parsování není známý předmět, ke kterému článek patří
	pub subject_card:      Option<SubjectCard>,
}

/// Karta předmětu
///
/// Tato struktura obsahuje informace z headeru
/// předmětu. Je využita pro generování profilu
/// předmětu a obsahuje základní informace.
///
/// Také využita v místech, kde není potřeba,
/// nebo není dostupný,
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectCard {
	/// název předmětu, bude použit ve všech odkazech
	/// a seznamech.
	pub nazev:            String,
	/// osoba zodpovědná za daný předmět,
	/// buď hlavní vyučující, nebo vyučující
	/// zodpovědný za dokumentaci (v případě více vyučujícíh)
	pub zodpovedna_osoba: String,
	/// krátký popisek předmětu
	pub bio:              String,
	/// cesta k předmětu,
	/// pro účely, kde je dostupná jenom karta
	/// předmětu nebo dokud není vytvořený profil
	/// předmětu
	pub _resolved_path:   Option<PathBuf>,
}

/// Předmět
///
/// Tato struktura reprezentuje celý profil předmětu,
/// využitý pro sestavování hierarchie knihy,
/// kategorizace článků a podobně.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
	/// karta předmětu
	pub card:            SubjectCard,
	/// cesta k předmětu
	pub path:            PathBuf,
	/// kořen předmětu,
	/// všechny `mdbook` kapitoly,
	/// jejichž cesta má `path_root` jako
	/// prefix jsou považovány za články
	/// spadající pod tento předmět
	pub path_root:       PathBuf,
	/// články spadající pod tento předmět
	pub articles:        Vec<Article>,
	/// pokud se podařilo vyřešit identitu
	/// zodpovědné osoby, zde je uložena
	/// její karta
	pub resolved_author: Option<TeacherCard>,
}

/// Učitel
///
/// Tato struktura obsahuje kompletní profil učitele.
/// Jedná se o stěžejní strukturu pro generování výstupu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
	/// karta učitele
	pub card:          TeacherCard,
	/// předměty, na kterých se
	/// vyučující podílel
	pub subjects:      Vec<Subject>,
	/// články, které vyučující založil
	pub articles:      Vec<Article>,
	/// seznam souborů, které uživatel přidal
	/// do gitu a stále existují
	pub files_created: Vec<PathBuf>,
}
