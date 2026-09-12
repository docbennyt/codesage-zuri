use anyhow::Result;
use crossterm::{event::{self,Event,KeyCode},execute,terminal::{disable_raw_mode,enable_raw_mode,EnterAlternateScreen,LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend,layout::{Constraint,Direction,Layout},widgets::{Block,Borders,List,ListItem,Paragraph,Wrap},Terminal};
use std::{io,path::Path,time::Duration};

pub fn run(root:&Path)->Result<()> {
    let db=zuri_core::open_project(root)?;
    if db.stats()?.files==0 { anyhow::bail!("project has no index yet; run `zuri index .` before opening the TUI"); }
    let symbols=db.symbols(None)?; let findings=db.findings()?;
    enable_raw_mode()?; let mut stdout=io::stdout(); execute!(stdout,EnterAlternateScreen)?;
    let backend=CrosstermBackend::new(stdout); let mut terminal=Terminal::new(backend)?;
    let result=(||->Result<()> {
        let mut selected=0usize;
        loop {
            terminal.draw(|f| {
                let outer=Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3),Constraint::Min(5)]).split(f.area());
                let stats=db.stats().unwrap_or_default();
                f.render_widget(Paragraph::new(format!("CodeSage Zuri | {} | Python | Offline/no-model | {} findings",root.display(),stats.findings)).block(Block::default().borders(Borders::ALL)),outer[0]);
                let cols=Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(32),Constraint::Percentage(38),Constraint::Percentage(30)]).split(outer[1]);
                let items:Vec<ListItem>=symbols.iter().enumerate().map(|(i,s)|ListItem::new(format!("{} {}",if i==selected{">"}else{" "},s.qualified_name))).collect();
                f.render_widget(List::new(items).block(Block::default().title(" Symbols ↑↓ ").borders(Borders::ALL)),cols[0]);
                let detail=symbols.get(selected).map(|s|format!("{}\n{}:{}-{}\n\n{}\n\nbranches: {}\nloops: {}\nreturns: {}\n\nconcepts:\n{}",s.qualified_name,s.location.file,s.location.start_line,s.location.end_line,s.signature,s.metrics.branches,s.metrics.loops,s.metrics.returns,s.concepts.join("\n"))).unwrap_or_else(||"No symbols indexed.".into());
                f.render_widget(Paragraph::new(detail).block(Block::default().title(" Inspect ").borders(Borders::ALL)).wrap(Wrap{trim:false}),cols[1]);
                let text=findings.iter().take(20).map(|x|format!("{} {}:{}\n{}",x.rule_id,x.location.file,x.location.start_line,x.title)).collect::<Vec<_>>().join("\n\n");
                f.render_widget(Paragraph::new(text).block(Block::default().title(" Findings | q quit ").borders(Borders::ALL)).wrap(Wrap{trim:false}),cols[2]);
            })?;
            if event::poll(Duration::from_millis(150))? { if let Event::Key(k)=event::read()? { match k.code { KeyCode::Char('q')|KeyCode::Esc=>break,KeyCode::Down=>if selected+1<symbols.len(){selected+=1},KeyCode::Up=>selected=selected.saturating_sub(1),_=>{} } } }
        } Ok(())
    })();
    disable_raw_mode()?; execute!(terminal.backend_mut(),LeaveAlternateScreen)?; terminal.show_cursor()?; result
}
