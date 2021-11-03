const PORT = Cypress.env('PORT') || '3000';

describe('MathLingua Editor', () => {
  beforeEach(() => {
    cy.visit(`http://localhost:${PORT}`);
    // switch to reader mode and not edit mode
    cy.get('[data-test-id="edit-mode-button"]').click();
  });

  it('shows the first page initially', () => {
    cy.contains('Introduction');
    cy.contains('This is some text that is in the introduction.');
  });

  it('can switch to different pages', () => {
    cy.contains('Some Page').click();
    cy.contains('Some text on Some_Page.math.');
  });

  it('can expand top level group items and then close them', () => {
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
