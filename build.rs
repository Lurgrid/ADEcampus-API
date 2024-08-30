use cfgrammar::yacc::YaccKind;
use lrlex::CTLexerBuilder;

const YACC_F: &'static str = "ical/ical.y";
const LEX_F: &'static str = "ical/ical.l";

fn main() {
    CTLexerBuilder::new()
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .recoverer(lrpar::RecoveryKind::None)
                .grammar_in_src_dir(YACC_F)
                .unwrap()
        })
        .lexer_in_src_dir(LEX_F)
        .unwrap()
        .build()
        .unwrap();
}
