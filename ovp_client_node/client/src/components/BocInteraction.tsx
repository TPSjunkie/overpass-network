import React, { useState, type ReactNode } from 'react';
import { useTonConnectUI } from "@tonconnect/ui-react";
import {
  parseBoc,
  serializeBoc,
  verifyBoc,
  extractCellsFromBoc,
} from "../utils/bocutils";

interface ContainerProps {
  children: ReactNode;
  className?: string;
}

const Container: React.FC<ContainerProps> = ({ children, className }) => (
  <div className={`container ${className || ''}`}>{children}</div>
);

const Row: React.FC<{ children: ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`row ${className || ''}`}>{children}</div>
);

const Col: React.FC<{ children: ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`col ${className || ''}`}>{children}</div>
);

interface FormProps {
  onSubmit: (e: React.FormEvent<HTMLFormElement>) => void;
  children: ReactNode;
}

const Form: React.FC<FormProps> = ({ onSubmit, children }) => (
  <form onSubmit={onSubmit}>{children}</form>
);

interface ButtonProps {
  type?: "button" | "submit" | "reset";
  onClick?: () => void;
  children: ReactNode;
  variant?: string;
}

const Button: React.FC<ButtonProps> = ({ type = "button", onClick, children, variant }) => (
  <button type={type} onClick={onClick} className={`btn btn-${variant || 'primary'}`}>{children}</button>
);

interface AlertProps {
  variant: string;
  children: ReactNode;
  className?: string;
}

const Alert: React.FC<AlertProps> = ({ variant, children, className }) => (
  <div className={`alert alert-${variant} ${className || ''}`}>{children}</div>
);

const BocInteraction = () => {
  const [bocData, setBocData] = useState('');
  const [result, setResult] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  const handleBocDataChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setBocData(e.target.value);
  };

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setResult(null);

    try {
      const parsedBoc = parseBoc(Buffer.from(bocData));
      const verificationResult = verifyBoc(Buffer.from(bocData));
      const extractedCells = extractCellsFromBoc(Buffer.from(bocData));
      const serializedBoc = serializeBoc(parsedBoc);

      setResult({
        parsedBoc,
        verificationResult,
        extractedCells,
        serializedBoc,
      });
    } catch (err) {
      setError((err as Error).message);
    }
  };

  return (
    <div className="boc-interaction bg-pip-boy-panel p-4 rounded-lg shadow-pip-boy">
    <h3 className="text-xl font-semibold text-pip-boy-green mb-2">BOC Interaction</h3>
    <p className="text-pip-boy-text">Interacting with BOC...</p>
 <div className="boc-interaction-form">
      <Form onSubmit={handleSubmit}>
        <Container>
          <Row>
            <Col>
              <label htmlFor="boc-data" className="form-label">
                BOC Data
              </label>
              <textarea
                id="boc-data"
                className="form-control"
                value={bocData}
                onChange={handleBocDataChange}
                rows={10}
              />
            </Col>
            <Col>
              <Button type="submit" variant="primary">
                Parse
              </Button>
            </Col>
          </Row>
        </Container>
      </Form>
      {error && <Alert variant="danger">{error}</Alert>}
      {result && (
        <div className="result-container">
          <h4 className="text-pip-boy-green">Result</h4>
          <pre className="result-pre">{JSON.stringify(result, null, 2)}</pre>
        </div>
      )}
    </div>
  </div>
);
};

export default BocInteraction;