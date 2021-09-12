describe('MathLingua Editor', () => {
  beforeEach(() => {
    cy.visit('http://localhost:3000');
  });

  it('shows the first page initially', () => {
    cy.contains('Introduction');
    cy.contains('This is some text that is in the introduction.');
  });

  it('can switch to different pages', () => {
    cy.contains('Some Page').click();
    cy.contains('Some text on Some_Page.math.');
  });

  it('displays a popup menu when clicking on a statement', () => {
    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function');
  });

  it('displays a panel when clicking a statement popup menu item', () => {
    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function').click();
    cy.contains('SomeFunctionCalled');
  });

  it("closes a top-level-entry in a sub-panel when it's close button is pressed", () => {
    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function').click();
    cy.contains('SomeFunctionCalled');
    cy.get('[data-test-id="close-single-top-level-entry"]').click();
    cy.contains('SomeFunctionCalled').should('not.exist');
  });

  it('closes all top-level-entries in a sub-panel when the close all button is pressed', () => {
    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function').click();

    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function').click();

    cy.get('.mathlingua-statement-container').click();
    cy.contains('\\some.function').click();

    cy.get('[data-test-id="close-all-entities"]').click({ force: true });
    cy.contains('SomeFunctionCalled').should('not.exist');
  });
});

export {};
